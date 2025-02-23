#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]

use std::any::{Any, TypeId};
use std::boxed::Box;
use std::collections::HashMap;
use std::pin::Pin;

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
        msg: Box<dyn Any>,
    },
    RegisterHandler {
        topic: String,
        // The closure is executed with a mutable reference to the MessageBus.
        registration: Box<dyn Fn(String) -> Pin<Box<ActorCoroutine>>>,
    },
}

/// For simplicity, we require that all our actor coroutines take () as input,
/// yield a Command, and return (). (This matches the coroutine trait signature in our snippet.)
pub type ActorCoroutine = dyn Coroutine<(), Yield = Command, Return = ()>;

///////////////////////////////////////////////////////////////////////////////
// We now define a typemap for coroutine handlers.
// When registering a handler for a message type M, we store a collection
// of functions of type: Fn(M) -> Pin<Box<ActorCoroutine>>. When a message is sent,
// we retrieve the appropriate collection and (for each handler) call it to create
// a new coroutine task for processing the message.

/// A concrete collection of coroutine handlers for messages of type M.
#[derive(Default)]
pub struct CoroutineHandlerCollection<M: 'static + Clone> {
    handlers: Vec<Box<dyn Fn(M) -> Pin<Box<ActorCoroutine>>>>,
}

impl<M: 'static + Clone> CoroutineHandlerCollection<M> {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn add_handler(&mut self, handler: Box<dyn Fn(M) -> Pin<Box<ActorCoroutine>>>) {
        self.handlers.push(handler);
    }

    /// Create a coroutine for each registered handler.
    pub fn call_handlers(&self, msg: M) -> Vec<Pin<Box<ActorCoroutine>>> {
        self.handlers.iter().map(|h| h(msg.clone())).collect()
    }
}

///////////////////////////////////////////////////////////////////////////////
// The message bus maintains a typemap of registered handlers (keyed by topic,
// and then by the message type's TypeId) and a stack of active coroutines.
pub struct MessageBus {
    // For each topic we map the message's TypeId to a handler collection.
    handlers: HashMap<String, CoroutineHandlerCollection<String>>,
    // A stack of active (suspended) coroutine tasks.
    task_stack: Vec<Pin<Box<ActorCoroutine>>>,
}

impl MessageBus {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            task_stack: Vec::new(),
        }
    }

    /// Registers a coroutine handler for message type M on the given topic.
    pub fn register_handler(
        &mut self,
        topic: &str,
        handler: Box<dyn Fn(String) -> Pin<Box<ActorCoroutine>>>,
    ) {
        let collection = self
            .handlers
            .entry(topic.to_string())
            .or_insert_with(|| CoroutineHandlerCollection::<String>::default());

        collection.add_handler(handler);
    }

    /// Sends a message of type M to the given topic. This creates new coroutine
    /// tasks for each registered handler, pushes them onto the stack, and then runs the bus.
    pub fn send(&mut self, topic: &str, msg: String) {
        if let Some(topic_map) = self.handlers.get(topic) {
            let coroutines = topic_map.call_handlers(msg);
            for coro in coroutines {
                self.task_stack.push(coro);
            }
        }
        self.run();
    }

    /// A helper to send a dynamically–typed message (used for processing yielded commands).
    fn send_dynamic(&mut self, topic: &str, msg: Box<dyn Any>) {
        if let Some(topic_map) = self.handlers.get(topic) {
            let msg = msg.downcast::<String>().unwrap();
            let coroutines = topic_map.call_handlers(*msg.clone());
            for coro in coroutines {
                self.task_stack.push(coro);
            }
        }
    }

    /// Run the bus until no suspended coroutine tasks are left.
    /// Whenever a coroutine yields a command, we process it immediately (depth-first)
    /// before resuming the coroutine.
    fn run(&mut self) {
        while let Some(mut coro) = self.task_stack.pop() {
            match Pin::new(&mut coro).resume(()) {
                CoroutineState::Yielded(cmd) => {
                    // Process the yielded command.
                    match cmd {
                        Command::Send { topic, msg } => {
                            self.send_dynamic(&topic, msg);
                        }
                        Command::RegisterHandler {
                            topic,
                            registration,
                        } => {
                            self.register_handler(&topic, registration);
                        }
                    }
                    // The coroutine is not done; push it back to resume later.
                    self.task_stack.push(coro);
                }
                CoroutineState::Complete(()) => {
                    // Coroutine finished.
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
        bus.register_handler(
            "start_topic",
            Box::new(|msg: String| {
                Box::pin(
                    #[coroutine]
                    move || {
                        if msg == "start" {
                            yield Command::RegisterHandler {
                                topic: "print_topic".to_string(),
                                registration: Box::new(|msg: String| {
                                    // Dynamically register a bool handler on "print_topic"
                                    Box::pin(
                                        #[coroutine]
                                        move || {
                                            println!("Dynamic bool handler received: {}", msg);
                                        },
                                    )
                                }),
                            };
                            // Then send a boolean message.
                            yield Command::Send {
                                topic: "print_topic".to_string(),
                                msg: Box::new("hello world".to_string()),
                            };
                        }
                        // Coroutine completes.
                    },
                )
            }),
        );

        // For illustration, also register a base bool handler on "print_topic".
        bus.register_handler(
            "print_topic",
            Box::new(|msg: String| {
                Box::pin(
                    #[coroutine]
                    move || {
                        println!("Default bool handler received: {}", msg);
                    },
                )
            }),
        );

        // Send a "start" message; it triggers the start coroutine,
        // which yields commands to register a new bool handler and send a bool message.
        bus.send("start_topic", "start".to_string());

        // Send another bool message to show that both the default and dynamic bool handlers are now registered.
        bus.send("print_topic", "hello world".to_string());
    }
}
