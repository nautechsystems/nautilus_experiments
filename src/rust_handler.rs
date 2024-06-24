use std::{any::Any, cell::RefCell, rc::Rc};

use pyo3::prelude::*;

use crate::{ChainEvent, MessageBusContext, MessageHandler, SendEvent};

#[pyo3::pyclass]
#[derive(Clone)]
pub struct RustMessageHandlerWrapper {
    pub handler: Rc<RefCell<RustMessageHandler>>,
}

unsafe impl Send for RustMessageHandlerWrapper {}

impl From<RustMessageHandler> for RustMessageHandlerWrapper {
    fn from(handler: RustMessageHandler) -> Self {
        RustMessageHandlerWrapper {
            handler: Rc::new(RefCell::new(handler)),
        }
    }
}

#[pymethods]
impl RustMessageHandlerWrapper {
    #[new]
    fn new(id: usize, context: MessageBusContext) -> Self {
        RustMessageHandlerWrapper::from(RustMessageHandler::new(id, context))
    }

    fn id(&self) -> usize {
        self.handler.borrow().id
    }

    fn get_data(&self) -> Vec<usize> {
        self.handler.borrow().get_data()
    }

    fn send(&self, id: usize, data: usize) {
        self.handler
            .borrow()
            .context
            .send(id, Box::new(SendEvent { data }));
    }

    fn chain(&self, data: usize) {
        self.handler.borrow().context.send(
            self.handler.borrow().id,
            Box::new(ChainEvent {
                data,
                start: self.handler.borrow().id,
            }),
        );
    }
}

pub struct RustMessageHandler {
    id: usize,
    data: Vec<usize>,
    context: MessageBusContext,
}

impl RustMessageHandler {
    fn new(id: usize, context: MessageBusContext) -> Self {
        RustMessageHandler {
            id,
            data: Vec::new(),
            context,
        }
    }

    pub fn get_data(&self) -> Vec<usize> {
        self.data.clone()
    }
}

impl MessageHandler for RustMessageHandler {
    fn handle(&mut self, message: &dyn Any) {
        println!("Rust message handler: {}", self.id());
        if message.is::<SendEvent>() {
            println!("Received send event");
            if let Some(SendEvent { data }) = message.downcast_ref::<SendEvent>() {
                self.data.push(*data);
            }
        } else if message.is::<ChainEvent>() {
            println!("Received chain event");
            if let Some(ChainEvent { data, start }) = message.downcast_ref::<ChainEvent>() {
                self.data.push(*data);

                let mut next = self.id + 1;

                if !self.context.check_handler(next) {
                    next = 0;
                }

                if next != *start {
                    println!("Sending chain event to {}", next);
                    self.context.send(
                        next,
                        Box::new(ChainEvent {
                            data: *data,
                            start: *start,
                        }),
                    );
                }
            }
        }
    }

    fn id(&self) -> usize {
        self.id
    }
}
