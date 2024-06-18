use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

trait MessageHandler {
    fn handle(&mut self, message: &dyn Any);
}

struct MessageBusContext {
    /// Hashmap mapping strings to handlers
    handlers: HashMap<String, Box<dyn MessageHandler>>,
}

impl MessageBusContext {
    fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    fn register_handler(&mut self, name: String, handler: Box<dyn MessageHandler>) {
        self.handlers.insert(name, handler);
    }

    fn handle_message(&mut self, name: &str, message: Box<dyn Any>) {
        if let Some(handler) = self.handlers.get_mut(name) {
            handler.handle(message.as_ref());
        }
    }
}

struct MessageEvent {
    name: String,
}

#[derive(Debug)]
struct DataBuilderMessageHandler {
    data: Vec<String>,
}

impl MessageHandler for DataBuilderMessageHandler {
    fn handle(&mut self, message: &dyn Any) {
        assert!(message.is::<MessageEvent>());

        if let Some(MessageEvent { name }) = message.downcast_ref::<MessageEvent>() {
            self.data.push(name.clone())
        }
    }
}

fn main() {
    let mut context = MessageBusContext::new();
    let data = Vec::new();
    let data_builder = DataBuilderMessageHandler { data };
    context.register_handler("data_builder".to_string(), Box::new(data_builder));

    context.handle_message(
        "data_builder",
        Box::new(MessageEvent {
            name: "hello".to_string(),
        }),
    );
    context.handle_message(
        "data_builder",
        Box::new(MessageEvent {
            name: "world".to_string(),
        }),
    );

    {
        let handler = context.handlers.get("data_builder").unwrap().as_ref();
        let inner_state: &DataBuilderMessageHandler =
            unsafe { &*(handler as *const dyn MessageHandler as *const DataBuilderMessageHandler) };
        assert_eq!(inner_state.data, vec!["hello", "world"]);
    }

    context.handle_message(
        "data_builder",
        Box::new(MessageEvent {
            name: "again".to_string(),
        }),
    );

    {
        let handler = context.handlers.get("data_builder").unwrap().as_ref();
        let inner_state: &DataBuilderMessageHandler = unsafe {
            &*(handler as *const dyn MessageHandler as *const &DataBuilderMessageHandler)
        };
        assert_eq!(inner_state.data, vec!["hello", "world", "again"]);
    }
}
