use std::any::Any;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use futures::future::LocalBoxFuture;
use futures::FutureExt;

pub trait Actor {
    fn id(&self) -> &str;
}

pub trait Handler<M: 'static> {
    fn handle(&mut self, msg: M);
}

///////////////////////////////////////////////////////////////////////////////
// HandlerCollection is a simple typemap container for handlers of a particular
// message type \(M\). We assume the message \(M\) is Clone so that each registered
// handler can get its own copy.
pub struct HandlerCollection<M> {
    handlers: Vec<Box<dyn Fn(M) -> LocalBoxFuture<'static, ()>>>,
}

impl<M: 'static + Clone> HandlerCollection<M> {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn add_handler(&mut self, handler: Box<dyn Fn(M) -> LocalBoxFuture<'static, ()>>) {
        self.handlers.push(handler);
    }

    pub async fn call_handlers(&self, msg: M) {
        for handler in &self.handlers {
            // Run each handler synchronously.
            handler(msg.clone()).await;
        }
    }
}

/// A shared message bus for a single-threaded environment.
/// It holds both the actor registry and the handler closures.
struct MessageBus {
    // Global actor state store: maps actor id to the actor's state.
    actors: HashMap<String, Rc<dyn Any>>,
    // Handlers: for each topic and for each message type, a closure is stored.
    handlers: HashMap<String, HashMap<TypeId, Box<dyn Any>>>,
}

impl MessageBus {
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            actors: HashMap::new(),
            handlers: HashMap::new(),
        }))
    }

    // Register an actor by storing its state in the bus.
    fn register_actor(&mut self, id: &str, actor: Rc<dyn Any>) {
        self.actors.insert(id.to_string(), actor);
    }

    // Helper to retrieve an actor from the registry.
    fn get_actor<A: 'static>(&self, id: &str) -> Option<Rc<RefCell<A>>> {
        self.actors.get(id).map(|actor_any| {
            actor_any
                .clone()
                .downcast::<RefCell<A>>()
                .expect("Actor type mismatch")
        })
    }

    pub fn register<M: 'static + Clone>(
        &mut self,
        topic: &str,
        handler: Box<dyn Fn(M) -> LocalBoxFuture<'static, ()>>,
    ) {
        let topic_map = self
            .handlers
            .entry(topic.to_string())
            .or_insert_with(HashMap::new);
        let type_id = TypeId::of::<M>();
        if let Some(collection_any) = topic_map.get_mut(&type_id) {
            let collection = collection_any
                .downcast_mut::<HandlerCollection<M>>()
                .unwrap();
            collection.add_handler(handler);
        } else {
            let mut collection = HandlerCollection::<M>::new();
            collection.add_handler(handler);
            topic_map.insert(type_id, Box::new(collection));
        }
    }

    // Send a message of type \(M\) to a given topic.
    pub async fn send<M: 'static + Clone>(&self, topic: &str, msg: M) {
        if let Some(topic_map) = self.handlers.get(topic) {
            if let Some(collection_any) = topic_map.get(&TypeId::of::<M>()) {
                let collection = collection_any
                    .downcast_ref::<HandlerCollection<M>>()
                    .unwrap();
                collection.call_handlers(msg).await;
            }
        }
    }
}

/// Our actor, which implements logic for handling messages.
struct MyActor {
    msgbus: Rc<RefCell<MessageBus>>,
    true_count: usize,
    id: String,
}

impl Handler<bool> for MyActor {
    fn handle(&mut self, msg: bool) {
        self.true_count += if msg { 1 } else { 0 };
    }
}

impl Handler<String> for MyActor {
    fn handle(&mut self, msg: String) {
        match msg.as_str() {
            "start" => {
                self.msgbus.borrow_mut().send("bool_topic", true);
            }
            "stop" => {
                // Deregistration logic
            }
            _ => println!("Received: {}", msg),
        }
    }
}

impl MyActor {
    fn new(id: &str, msgbus: Rc<RefCell<MessageBus>>) -> Rc<RefCell<Self>> {
        // Create the actor.
        let actor = Rc::new(RefCell::new(Self {
            msgbus: msgbus.clone(),
            true_count: 0,
            id: id.to_string(),
        }));

        // Register itself in the actor store.
        msgbus
            .borrow_mut()
            .register_actor(id, actor.clone() as Rc<dyn Any>);
        actor
    }
}

impl Actor for MyActor {
    fn id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::LocalPool;

    use super::*;

    #[test]
    fn test_actor_framework_with_global_store() {
        let msgbus = MessageBus::new();
        let actor = MyActor::new("actor1", msgbus.clone());

        let handler = Box::new({
            let msgbus = msgbus.clone();
            move |msg: String| {
                let actor = msgbus.borrow().get_actor::<MyActor>("actor1").unwrap();
                async move { MyActor::handle(&mut actor.borrow_mut(), msg.clone()) }.boxed_local()
            }
        });

        // Register a String handler.
        msgbus
            .borrow_mut()
            .register::<String>("string_topic", handler);

        let mut pool = LocalPool::new();
        // Send "start", which sets up the bool handler.
        {
            pool.run_until(msgbus
                .borrow_mut()
                .send("string_topic", "start".to_string()));
        }

        // Send some boolean messages.
        {
            pool.run_until(msgbus.borrow_mut().send("bool_topic", true));
            pool.run_until(msgbus.borrow_mut().send("bool_topic", false));
            pool.run_until(msgbus.borrow_mut().send("bool_topic", true));
        }

        // Confirm that the actor's state was updated.
        {
            let actor_ref = actor.borrow();
            assert_eq!(actor_ref.true_count, 2);
        }
    }
}
