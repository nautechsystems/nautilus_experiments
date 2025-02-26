#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]
#![feature(trait_alias)]

use std::any::{Any, TypeId};
use std::boxed::Box;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::rc::Rc;

// We use futures' LocalBoxFuture only in earlier examples; here we use experimental coroutines.
// (The coroutine feature requires that you compile with nightly.)
// The Coroutine and CoroutineState traits (and its implementations) are provided by the compiler.
// (Their definitions are shown in your attached snippet.)

use core::ops::Coroutine;
use core::ops::CoroutineState;

/// A command that a coroutine can yield. Here we support sending a message and handler registration.
pub enum Command {
    Send {
        topic: String,
        // Boxed dynamic message.
        msg: Rc<dyn Any>,
    },
    Publish {
        pattern: String,
        msg: Rc<dyn Any>,
    },
    /// Register an endpoint subscription
    Register(Subscription),
    /// Deregister an endpoint subscription
    Deregister(String),
    /// Subscribe to a topic
    Subscribe(Subscription),
    /// Unsubscribe from a topic
    Unsubscribe((String, String)),
}

pub type ActorCoroutine = Pin<Box<dyn Coroutine<Rc<dyn Any>, Yield = Command, Return = ()>>>;
pub type ActorFn = Box<dyn Fn() -> ActorCoroutine>;

pub struct PublishTask {
    pattern: String,
    msg: Rc<dyn Any>,
    idx: usize,
}

impl Display for PublishTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "PublishTask(pattern: {}, idx: {})",
            self.pattern, self.idx
        )
    }
}

impl PublishTask {
    pub fn new(pattern: String, msg: Rc<dyn Any>) -> Self {
        Self {
            pattern,
            msg,
            idx: 0,
        }
    }

    // Dummy implementation
    pub fn next_task(&mut self, msg_bus: &MessageBus) -> Option<SendTask> {
        let sub = msg_bus
            .subscriptions
            .iter()
            .filter(|(_sub, pattern)| pattern.contains(&self.pattern))
            .map(|(sub, _)| sub)
            .nth(self.idx);

        sub.map(|sub| {
            self.idx += 1;
            println!(
                "publish to pattern {}, task id: {}, sub: {}",
                self.pattern, sub.handler_id, sub
            );
            let actor_fn = (sub.actor_fn)();
            Some(SendTask::new(
                self.pattern.clone(),
                actor_fn,
                self.msg.clone(),
            ))
        })
        .flatten()
    }
}

pub struct SendTask {
    pattern: String,
    coro: ActorCoroutine,
    msg: Rc<dyn Any>,
}

impl Display for SendTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "SendTask: {}", self.pattern)
    }
}

impl SendTask {
    pub fn new(pattern: String, coro: ActorCoroutine, msg: Rc<dyn Any>) -> Self {
        Self { pattern, coro, msg }
    }

    pub fn resume(&mut self) -> CoroutineState<Command, ()> {
        let msg = self.msg.clone();
        self.coro.as_mut().resume(msg)
    }
}

pub enum Task {
    Send(SendTask),
    Publish(PublishTask),
}

impl Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Task::Send(send) => writeln!(f, "{}", send),
            Task::Publish(publish) => writeln!(f, "{}", publish),
        }
    }
}

#[derive(Default)]
pub struct TaskRunner {
    pub tasks: Vec<Task>,
    pub msg_bus: MessageBus,
}

impl Display for TaskRunner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "TaskRunner")?;
        writeln!(f, "tasks:")?;
        for task in &self.tasks {
            writeln!(f, "{}", task)?;
        }
        Ok(())
    }
}

impl TaskRunner {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            msg_bus: MessageBus::new(),
        }
    }

    pub fn push(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn pop(&mut self) -> Option<Task> {
        self.tasks.pop()
    }

    pub fn run(&mut self) {
        while let Some(task) = self.tasks.last_mut() {
            match task {
                Task::Send(send) => {
                    match send.resume() {
                        CoroutineState::Yielded(cmd) => {
                            // Process the yielded command.
                            match cmd {
                                Command::Send { topic, msg } => {
                                    println!("Sending message");
                                    if let Some(sub) = self.msg_bus.endpoints.get(&topic) {
                                        let coro = (sub.actor_fn)();
                                        self.push(Task::Send(SendTask::new(topic, coro, msg)));
                                    }
                                }
                                Command::Register(subscription) => {
                                    println!("Registering handler");
                                    self.msg_bus.register(subscription);
                                }
                                Command::Deregister(topic) => {
                                    self.msg_bus.deregister(&topic);
                                }
                                Command::Subscribe(subscription) => {
                                    self.msg_bus.subscribe(subscription);
                                }
                                Command::Unsubscribe((topic, handler_id)) => {
                                    self.msg_bus.remove_subscription(&topic, &handler_id);
                                }
                                Command::Publish { pattern, msg } => {
                                    self.push(Task::Publish(PublishTask::new(pattern, msg)));
                                }
                            }
                        }
                        CoroutineState::Complete(_) => {
                            self.tasks.pop();
                        }
                    }
                }
                Task::Publish(publish) => match publish.next_task(&self.msg_bus) {
                    Some(send) => self.push(Task::Send(send)),
                    None => {
                        self.tasks.pop();
                    }
                },
            }
        }
    }
}

pub struct Subscription {
    /// The shareable message handler for the subscription.
    pub actor_fn: ActorFn,
    /// Store a copy of the handler ID for faster equality checks.
    pub handler_id: String,
    /// The topic for the subscription.
    pub topic: String,
    /// The priority for the subscription determines the ordering of handlers receiving
    /// messages being processed, higher priority handlers will receive messages before
    /// lower priority handlers.
    pub priority: u8,
}

impl Hash for Subscription {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.topic.hash(state);
        self.handler_id.hash(state);
    }
}

impl PartialEq for Subscription {
    fn eq(&self, other: &Self) -> bool {
        self.topic == other.topic && self.handler_id == other.handler_id
    }
}

impl Eq for Subscription {}

impl Display for Subscription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sub::{}:{}", self.topic, self.handler_id)
    }
}

#[derive(Default)]
pub struct MessageBus {
    endpoints: HashMap<String, Subscription>,
    subscriptions: HashMap<Subscription, String>,
}

impl Display for MessageBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Messagebus")?;
        writeln!(f, "endpoints:")?;
        for (topic, sub) in &self.endpoints {
            writeln!(f, "{}: {}", topic, sub)?;
        }
        writeln!(f, "subscriptions:")?;
        for (sub, topic) in &self.subscriptions {
            writeln!(f, "{}: {}", sub, topic)?;
        }
        Ok(())
    }
}

impl MessageBus {
    pub fn new() -> Self {
        Self {
            endpoints: HashMap::new(),
            subscriptions: HashMap::new(),
        }
    }

    /// Registers a coroutine handler for message type M on the given topic.
    pub fn register(&mut self, subscription: Subscription) {
        self.endpoints
            .insert(subscription.topic.clone(), subscription);
    }

    pub fn deregister(&mut self, topic: &str) {
        self.endpoints.remove(topic);
    }

    pub fn subscribe(&mut self, subscription: Subscription) {
        let topic = subscription.topic.clone();
        self.subscriptions.insert(subscription, topic);
    }

    pub fn remove_subscription(&mut self, topic: &str, handler_id: &str) {
        // create dummy subscription
        let key = Subscription {
            topic: topic.to_string(),
            handler_id: handler_id.to_string(),
            actor_fn: Box::new(|| {
                Box::pin(
                    #[coroutine]
                    |_: Rc<dyn Any>| {},
                )
            }), // dummy fn
            priority: 0,
        };
        self.subscriptions.remove(&key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Test 1: Register an endpoint, send messages, then deregister.
    #[test]
    fn test_register_deregister_and_send() {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        let mut bus = MessageBus::new();

        // Register an endpoint which increments our counter.
        bus.register(Subscription {
            topic: "endpoint_topic".to_string(),
            actor_fn: Box::new(move || {
                let counter = counter.clone();
                Box::pin(
                    #[coroutine]
                    move |_msg: Rc<dyn Any>| {
                        *counter.borrow_mut() += 1;
                    },
                )
            }),
            handler_id: "ep1".to_string(),
            priority: 0,
        });

        // Send a message and run.
        let task = Task::Send(SendTask::new(
            "endpoint_topic".to_string(),
            (bus.endpoints["endpoint_topic"].actor_fn)(),
            Rc::new(()),
        ));
        let mut runner = TaskRunner::new();
        runner.push(task);
        runner.run();
        assert_eq!(*counter_clone.borrow(), 1);

        // Now deregister the endpoint and try to send another message.
        bus.deregister("endpoint_topic");
        // Message won't be delivered since endpoint is gone
        assert_eq!(*counter_clone.borrow(), 1);
    }

    /// Test 2: Subscribe multiple handlers, send messages, unsubscribe one and test interleaving.
    #[test]
    fn test_subscribe_multiple_and_unsubscribe() {
        let counter1 = Rc::new(RefCell::new(0));
        let counter2 = Rc::new(RefCell::new(0));
        let sub_counter1 = counter1.clone();
        let sub_counter2 = counter2.clone();

        let mut runner = TaskRunner::new();

        // Register two subscriptions on the same topic.
        runner.msg_bus.subscribe(Subscription {
            topic: "pubsub_topic".to_string(),
            actor_fn: Box::new(move || {
                let value = sub_counter1.clone();
                Box::pin(
                    #[coroutine]
                    move |_msg: Rc<dyn Any>| {
                        *value.borrow_mut() += 1;
                    },
                )
            }),
            handler_id: "sub1".to_string(),
            priority: 0,
        });
        runner.msg_bus.subscribe(Subscription {
            topic: "pubsub_topic".to_string(),
            actor_fn: Box::new(move || {
                let mut value = sub_counter2.clone();
                Box::pin(
                    #[coroutine]
                    move |_msg: Rc<dyn Any>| {
                        *value.borrow_mut() += 1;
                    },
                )
            }),
            handler_id: "sub2".to_string(),
            priority: 0,
        });
        println!("{}", runner);

        // Send a message; both subscriptions should process it.
        runner.push(Task::Publish(PublishTask::new(
            "pubsub_topic".to_string(),
            Rc::new(()),
        )));
        println!("{}", runner);
        runner.run();
        println!("{}", runner);
        assert_eq!(*counter1.borrow(), 1);
        assert_eq!(*counter2.borrow(), 1);

        // Unsubscribe the first subscription.
        runner.msg_bus.remove_subscription("pubsub_topic", "sub1");

        // Send another message; only the second subscription should process it.
        runner.push(Task::Publish(PublishTask::new(
            "pubsub_topic".to_string(),
            Rc::new(()),
        )));
        runner.run();
        assert_eq!(*counter1.borrow(), 1);
        assert_eq!(*counter2.borrow(), 2);
    }
}
