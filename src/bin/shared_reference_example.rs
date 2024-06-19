use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

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

struct CommandEvent {
    op: String,
}

#[derive(Debug)]
struct DataBuilderMessageHandler {
    data: Rc<RefCell<Vec<String>>>,
}

impl MessageHandler for DataBuilderMessageHandler {
    fn handle(&mut self, message: &dyn Any) {
        assert!(message.is::<MessageEvent>());

        if let Some(MessageEvent { name }) = message.downcast_ref::<MessageEvent>() {
            self.data.borrow_mut().push(name.clone())
        }
    }
}

struct CommandMessageHandler {
    data: Rc<RefCell<Vec<String>>>,
}

impl MessageHandler for CommandMessageHandler {
    fn handle(&mut self, message: &dyn Any) {
        assert!(message.is::<CommandEvent>());

        if let Some(CommandEvent { op }) = message.downcast_ref::<CommandEvent>() {
            match op.as_str() {
                // Add match arms to handle all possible values of `name`
                "pop" => {
                    self.data.borrow_mut().pop();
                }
                "clear" => {
                    self.data.borrow_mut().clear();
                }
                _ => {}
            }
        }
    }
}

fn main() {
    let mut context = MessageBusContext::new();
    let data = Rc::new(RefCell::new(Vec::new()));
    let data_builder = DataBuilderMessageHandler { data: data.clone() };
    let command_handler = CommandMessageHandler { data };
    context.register_handler("data_builder".to_string(), Box::new(data_builder));
    context.register_handler("command".to_string(), Box::new(command_handler));

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

        // check that data was added
        assert_eq!(inner_state.data.borrow().as_ref(), vec!["hello", "world"]);
    }

    {
        let handler = context.handlers.get("command").unwrap().as_ref();
        let inner_state: &CommandMessageHandler =
            unsafe { &*(handler as *const dyn MessageHandler as *const CommandMessageHandler) };

        // check that data was added
        assert_eq!(inner_state.data.borrow().as_ref(), vec!["hello", "world"]);
    }

    context.handle_message(
        "command",
        Box::new(CommandEvent {
            op: "pop".to_string(),
        }),
    );

    {
        let handler = context.handlers.get("data_builder").unwrap().as_ref();
        let inner_state: &DataBuilderMessageHandler = unsafe {
            &*(handler as *const dyn MessageHandler as *const &DataBuilderMessageHandler)
        };

        // check that last element was popped
        assert_eq!(inner_state.data.borrow().as_ref(), vec!["hello"]);
    }

    {
        let handler = context.handlers.get("command").unwrap().as_ref();
        let inner_state: &CommandMessageHandler =
            unsafe { &*(handler as *const dyn MessageHandler as *const CommandMessageHandler) };

        // check that data was added
        assert_eq!(inner_state.data.borrow().as_ref(), vec!["hello", "world"]);
    }

    context.handle_message(
        "command",
        Box::new(CommandEvent {
            op: "clear".to_string(),
        }),
    );

    {
        let handler = context.handlers.get("data_builder").unwrap().as_ref();
        let inner_state: &DataBuilderMessageHandler = unsafe {
            &*(handler as *const dyn MessageHandler as *const &DataBuilderMessageHandler)
        };

        // check that last element was popped
        assert_eq!(inner_state.data.borrow().as_ref(), Vec::<String>::new());
    }

    {
        let handler = context.handlers.get("command").unwrap().as_ref();
        let inner_state: &CommandMessageHandler =
            unsafe { &*(handler as *const dyn MessageHandler as *const CommandMessageHandler) };

        // check that data was added
        assert_eq!(inner_state.data.borrow().as_ref(), Vec::<String>::new());
    }
}
