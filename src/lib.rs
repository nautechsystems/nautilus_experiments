use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

use pyo3::prelude::*;
use python_handler::PythonMessageHandler;
use rust_handler::RustMessageHandlerWrapper;

pub mod python_handler;
pub mod rust_handler;

pub trait MessageHandler {
    fn id(&self) -> usize;
    fn handle(&mut self, message: &dyn Any);
}

type MessageHandlerMapping = Rc<RefCell<HashMap<usize, Rc<RefCell<dyn MessageHandler>>>>>;

#[pyo3::pyclass]
#[derive(Clone)]
struct MessageBusContext {
    /// Hashmap mapping strings to handlers
    handlers: MessageHandlerMapping,
}

unsafe impl Send for MessageBusContext {}

#[pymethods]
impl MessageBusContext {
    #[new]
    pub fn new() -> Self {
        Self {
            handlers: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn register_python_handler(&mut self, id: usize, handler: PyObject) {
        let handler = Rc::new(RefCell::new(PythonMessageHandler::new(handler)));
        self.register_handler(id, handler);
    }

    pub fn register_rust_handler(&mut self, id: usize, wrapper: RustMessageHandlerWrapper) {
        self.register_handler(id, wrapper.handler.clone());
    }

    pub fn py_send(&self, id: usize, event: SendEvent) {
        self.send(id, Box::new(event));
    }

    pub fn py_chain(&self, id: usize, event: ChainEvent) {
        self.send(id, Box::new(event));
    }

    pub fn check_handler(&self, id: usize) -> bool {
        self.handlers.borrow().contains_key(&id)
    }
}

impl MessageBusContext {
    fn register_handler(&mut self, id: usize, handler: Rc<RefCell<dyn MessageHandler>>) {
        self.handlers.borrow_mut().insert(id, handler);
    }

    fn send(&self, id: usize, message: Box<dyn Any>) {
        if let Some(handler) = self.handlers.borrow().get(&id) {
            handler.borrow_mut().handle(message.as_ref());
        }
    }
}

#[pyo3::pyclass]
#[derive(Clone)]
struct SendEvent {
    #[pyo3(get)]
    pub data: usize,
}

#[pymethods]
impl SendEvent {
    #[new]
    pub fn new(data: usize) -> Self {
        SendEvent { data }
    }
}

#[pyo3::pyclass]
#[derive(Clone)]
struct ChainEvent {
    #[pyo3(get)]
    pub data: usize,
    #[pyo3(get)]
    pub start: usize,
}

#[pymethods]
impl ChainEvent {
    #[new]
    pub fn new(start: usize, data: usize) -> Self {
        ChainEvent { start, data }
    }
}

// export pyo3 module
#[pymodule]
pub fn nautilus_experiments(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SendEvent>()?;
    m.add_class::<ChainEvent>()?;
    m.add_class::<MessageBusContext>()?;
    m.add_class::<RustMessageHandlerWrapper>()?;
    Ok(())
}
