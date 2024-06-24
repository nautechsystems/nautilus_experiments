use std::any::Any;

use pyo3::prelude::*;

use crate::{ChainEvent, MessageHandler, SendEvent};

pub struct PythonMessageHandler {
    handler: PyObject,
}

impl PythonMessageHandler {
    pub fn new(handler: PyObject) -> Self {
        PythonMessageHandler { handler }
    }
}

impl MessageHandler for PythonMessageHandler {
    fn handle(&mut self, message: &dyn Any) {
        println!("Python message handler: {}", self.id());
        let py_event = if message.is::<SendEvent>() {
            println!("Received send event");
            Python::with_gil(|py| {
                message
                    .downcast_ref::<SendEvent>()
                    .map(|event| event.clone().into_py(py))
                    .unwrap()
            })
        } else if message.is::<ChainEvent>() {
            println!("Received chain event");
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
