use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

use pyo3::prelude::*;

trait MessageHandler {
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
        let handler = Rc::new(RefCell::new(PythonMessageHandler { handler }));
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
            println!("calling handler id: {}", handler.borrow().id());
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

struct PythonMessageHandler {
    handler: PyObject,
}

impl MessageHandler for PythonMessageHandler {
    fn handle(&mut self, message: &dyn Any) {
        let py_event = if message.is::<SendEvent>() {
            Python::with_gil(|py| {
                message
                    .downcast_ref::<SendEvent>()
                    .map(|event| event.clone().into_py(py))
                    .unwrap()
            })
        } else if message.is::<ChainEvent>() {
            Python::with_gil(|py| {
                message
                    .downcast_ref::<ChainEvent>()
                    .map(|event| event.clone().into_py(py))
                    .unwrap()
            })
        } else {
            eprintln!("Unknown message type: {:?}", message.type_id());
            return;
        };

        let result =
            pyo3::Python::with_gil(|py| self.handler.call_method1(py, "handle", (py_event,)));
        if let Err(err) = result {
            eprintln!("Error calling handle method: {:?}", err);
        }
    }

    fn id(&self) -> usize {
        Python::with_gil(|py| {
            self.handler
                .call_method0(py, "id")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }
}

#[pyo3::pyclass]
#[derive(Clone)]
struct RustMessageHandlerWrapper {
    handler: Rc<RefCell<RustMessageHandler>>,
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

struct RustMessageHandler {
    id: usize,
    data: Vec<usize>,
    context: MessageBusContext,
}

impl RustMessageHandler {
    pub fn new(id: usize, context: MessageBusContext) -> Self {
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
        if message.is::<SendEvent>() {
            if let Some(SendEvent { data }) = message.downcast_ref::<SendEvent>() {
                self.data.push(*data);
            }
        } else if message.is::<ChainEvent>() {
            if let Some(ChainEvent { data, start }) = message.downcast_ref::<ChainEvent>() {
                self.data.push(*data);

                let next = self.id + 1;

                match (next != *start, self.context.check_handler(next)) {
                    // send to next if it exists
                    (true, true) => {
                        self.context.send(
                            next,
                            Box::new(ChainEvent {
                                data: *data,
                                start: *start,
                            }),
                        );
                    }
                    // wrap around if next doesn't exist
                    (true, false) => {
                        self.context.send(
                            0,
                            Box::new(ChainEvent {
                                data: *data,
                                start: *start,
                            }),
                        );
                    }
                    // continue chain is next node is not start node
                    (false, _) => {}
                }
            }
        }
    }

    fn id(&self) -> usize {
        self.id
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
