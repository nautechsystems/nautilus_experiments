#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]
#![feature(trait_alias)]

use std::any::{Any, TypeId};
use std::boxed::Box;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
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

pub struct Handler {
    coro: ActorCoroutine,
    msg: Rc<dyn Any>,
}

impl Handler {
    pub fn new(coro: ActorCoroutine, msg: Rc<dyn Any>) -> Self {
        Self { coro, msg }
    }

    pub fn resume(&mut self) -> CoroutineState<Command, ()> {
        let msg = self.msg.clone();
        self.coro.as_mut().resume(msg)
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

impl Display for Subscription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sub::{}:{}", self.topic, self.handler_id)
    }
}

impl From<ActorFn> for Subscription {
    fn from(handler_fn: ActorFn) -> Self {
        Self {
            actor_fn: handler_fn,
            handler_id: "".to_string(),
            topic: "".to_string(),
            priority: 0,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// We now define a typemap for coroutine handlers.
// When registering a handler for a message type M, we store a collection
// of functions of type: Fn(M) -> Pin<Box<ActorCoroutine>>. When a message is sent,
// we retrieve the appropriate collection and (for each handler) call it to create
// a new coroutine task for processing the message.

///////////////////////////////////////////////////////////////////////////////
// The message bus maintains a typemap of registered handlers (keyed by topic,
// and then by the message type's TypeId) and a stack of active coroutines.
pub struct MessageBus {
    // For each topic we map the message's TypeId to a handler collection.
    endpoints: HashMap<String, Subscription>,
    // A stack of active (suspended) coroutine tasks.
    task_stack: Vec<Handler>,
}

impl MessageBus {
    pub fn new() -> Self {
        Self {
            endpoints: HashMap::new(),
            task_stack: Vec::new(),
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

    /// Sends a message of type M to the given topic. This creates new coroutine
    /// task for the endpoint handler and pushes it onto the stack.
    pub fn send(&mut self, topic: &str, msg: Rc<dyn Any>) {
        if let Some(subscription) = self.endpoints.get(topic) {
            let coro = (subscription.actor_fn)();
            self.task_stack.push(Handler::new(coro, msg));
        } else {
            panic!("No subscription found for topic: {}", topic);
        }
    }

    pub fn push(&mut self, handler: Handler) {
        self.task_stack.push(handler);
    }

    pub fn pop(&mut self) -> Option<Handler> {
        self.task_stack.pop()
    }

    /// Run the bus until no suspended coroutine tasks are left.
    /// Whenever a coroutine yields a command, we process it immediately (depth-first)
    /// before resuming the coroutine.
    pub fn run(&mut self) {
        while let Some(handler) = self.task_stack.last_mut() {
            match handler.resume() {
                CoroutineState::Yielded(cmd) => {
                    // Process the yielded command.
                    match cmd {
                        Command::Send { topic, msg } => {
                            println!("Sending message");
                            self.send(&topic, msg);
                        }
                        Command::Register(subscription) => {
                            println!("Registering handler");
                            self.register(subscription);
                        }
                        Command::Deregister(topic) => {
                            self.deregister(&topic);
                        }
                        _ => {}
                    }
                }
                CoroutineState::Complete(_) => {
                    self.task_stack.pop();
                }
            }
        }
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
