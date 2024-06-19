use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

trait MessageHandler {
    fn handle(&mut self, message: &dyn Any);
}

struct MessageBusContext {
    /// Hashmap mapping strings to handlers
    handlers: HashMap<String, Box<RefCell<dyn MessageHandler>>>,
}

impl MessageBusContext {
    fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    fn register_handler(&mut self, name: String, handler: Box<RefCell<dyn MessageHandler>>) {
        self.handlers.insert(name, handler);
    }

    fn handle_message(&self, name: &str, message: Box<dyn Any>) {
        if let Some(handler) = self.handlers.get(name) {
            handler.borrow_mut().handle(message.as_ref());
        }
    }
}

struct MessageEvent {
    name: String,
}

struct CountEvent {
    value: usize,
}

struct DataBuilderMessageHandler {
    data: Vec<String>,
    context: Rc<RefCell<MessageBusContext>>,
}

impl MessageHandler for DataBuilderMessageHandler {
    fn handle(&mut self, message: &dyn Any) {
        assert!(message.is::<MessageEvent>());

        if let Some(MessageEvent { name }) = message.downcast_ref::<MessageEvent>() {
            self.data.push(name.clone());

            // send count to a different handler
            self.context
                .borrow()
                .handle_message("count", Box::new(CountEvent { value: name.len() }));
        }
    }
}

struct CountHandler {
    data: Vec<usize>,
}

impl MessageHandler for CountHandler {
    fn handle(&mut self, message: &dyn Any) {
        assert!(message.is::<CountEvent>());

        if let Some(CountEvent { value }) = message.downcast_ref::<CountEvent>() {
            self.data.push(*value);
        }
    }
}

fn main() {
    let context = Rc::new(RefCell::new(MessageBusContext::new()));
    let data = Vec::new();
    let data_builder = DataBuilderMessageHandler {
        data,
        context: context.clone(),
    };
    let count_handler = CountHandler { data: Vec::new() };

    // register handlers
    context.borrow_mut().register_handler(
        "data_builder".to_string(),
        Box::new(RefCell::new(data_builder)),
    );
    context
        .borrow_mut()
        .register_handler("count".to_string(), Box::new(RefCell::new(count_handler)));

    // send messages
    context.borrow().handle_message(
        "data_builder",
        Box::new(MessageEvent {
            name: "hello".to_string(),
        }),
    );
    context.borrow().handle_message(
        "data_builder",
        Box::new(MessageEvent {
            name: "world".to_string(),
        }),
    );

    // test internal state
    {
        let context = context.borrow();
        let handler = context.handlers.get("data_builder").unwrap().as_ref();
        let inner_state: &DataBuilderMessageHandler = unsafe {
            &*(handler.as_ptr() as *const dyn MessageHandler as *const DataBuilderMessageHandler)
        };

        // check that data was added
        assert_eq!(inner_state.data.as_ref(), vec!["hello", "world"]);
    }

    {
        let context = context.borrow();
        let handler = context.handlers.get("count").unwrap().as_ref();
        let inner_state: &CountHandler =
            unsafe { &*(handler.as_ptr() as *const dyn MessageHandler as *const CountHandler) };

        // check that data was added
        assert_eq!(inner_state.data.as_ref(), vec![5, 5]);
    }
}
