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
            .take(self.idx)
            .last();

        if sub.is_none() {
            None
        } else {
            self.idx += 1;
            let actor_fn = (sub.unwrap().actor_fn)();
            Some(SendTask::new(actor_fn, self.msg.clone()))
        }
    }
}

pub struct SendTask {
    coro: ActorCoroutine,
    msg: Rc<dyn Any>,
}

impl SendTask {
    pub fn new(coro: ActorCoroutine, msg: Rc<dyn Any>) -> Self {
        Self { coro, msg }
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

#[derive(Default)]
pub struct TaskRunner {
    tasks: Vec<Task>,
    msg_bus: MessageBus,
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
                                        self.push(Task::Send(SendTask::new(coro, msg)));
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

///////////////////////////////////////////////////////////////////////////////
// Below is a test showing how handlers can be implemented as coroutines.
// In this example the "start_topic" handler (for String messages) yields commands
// to send Boolean messages; the Boolean handler simply prints the message.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_bus_with_coroutines_and_dynamic_registration() {
        let mut bus = MessageBus::new();

        // Register a handler for String messages on "start_topic".
        // This coroutine yields a RegisterHandler command, which registers a bool handler.
        bus.register(Subscription {
            topic: "start_topic".to_string(),
            actor_fn: Box::new(|| {
                Box::pin(
                    #[coroutine]
                    move |msg: Rc<dyn Any>| {
                        let msg = msg.downcast_ref::<String>().unwrap();
                        if msg == "start" {
                            yield Command::Register(Subscription {
                                topic: "print_topic".to_string(),
                                actor_fn: Box::new(|| {
                                    // Dynamically register a bool handler on "print_topic"
                                    Box::pin(
                                        #[coroutine]
                                        move |msg: Rc<dyn Any>| {
                                            let msg = msg.downcast_ref::<String>().unwrap();
                                            println!("Dynamic bool handler received: {}", msg);
                                        },
                                    )
                                }),
                                handler_id: "dynamic_bool_handler".to_string(),
                                priority: 0,
                            });
                            // Then send a boolean message.
                            yield Command::Send {
                                topic: "print_topic".to_string(),
                                msg: Rc::new("hello world".to_string()),
                            };
                        }
                        // Coroutine completes.
                    },
                )
            }),
            handler_id: "start_topic_handler".to_string(),
            priority: 0,
        });
        bus.send("start_topic", Rc::new("start".to_string()));
        bus.run();
        println!(
            "endpoints: {}",
            bus.endpoints
                .values()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        // For illustration, also register a base bool handler on "print_topic".
        bus.register(Subscription {
            topic: "print_topic".to_string(),
            actor_fn: Box::new(|| {
                Box::pin(
                    #[coroutine]
                    move |msg: Rc<dyn Any>| {
                        let msg = msg.downcast_ref::<String>().unwrap();
                        println!("Default bool handler received: {}", msg);
                    },
                )
            }),
            handler_id: "print_topic_handler".to_string(),
            priority: 0,
        });
        bus.send("print_topic", Rc::new("hello world".to_string()));
        bus.run();
        println!(
            "endpoints: {}",
            bus.endpoints
                .values()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        // Send a "start" message; it triggers the start coroutine,
        // which yields commands to register a new bool handler and send a bool message.
        bus.send("start_topic", Rc::new("start".to_string()));
        bus.run();
        println!(
            "endpoints: {}",
            bus.endpoints
                .values()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        // Send another bool message to show that both the default and dynamic bool handlers are now registered.
        bus.send("print_topic", Rc::new("hello world".to_string()));
        bus.run();
    }
}
