use std::any::Any;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

trait Actor {
    fn id(&self) -> &str;
}

trait Handler<M: 'static> {
    fn handle(&mut self, msg: M);
}

struct MyActor {
    msgbus: SharedMessageBus,
    true_count: usize,
}

impl MyActor {
    fn new(msgbus: SharedMessageBus) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            msgbus,
            true_count: 0,
        }))
    }
}

impl Actor for MyActor {
    fn id(&self) -> &str {
        "my_actor"
    }
}

impl Handler<String> for MyActor {
    fn handle(&mut self, msg: String) {
        match msg.as_str() {
            "start" => {
                let actor_ref = Rc::new(RefCell::new(self));
                let weak_ref = Rc::downgrade(&actor_ref);
                
                self.msgbus.borrow_mut().register::<bool>(
                    "bool_topic",
                    Box::new(move |msg| {
                        if let (Some(actor), Some(msg)) = (weak_ref.upgrade(), msg.downcast_ref::<bool>()) {
                            actor.borrow_mut().handle(*msg);
                        }
                    })
                );
            }
            "stop" => {
                // Deregistration logic
            }
            _ => println!("Received: {}", msg),
        }
    }
}

impl Handler<bool> for MyActor {
    fn handle(&mut self, msg: bool) {
        if msg {
            self.true_count += 1;
        }
    }
}

type SharedMessageBus = Rc<RefCell<MessageBus>>;
type BoxedHandler<M> = Box<dyn Handler<M>>;

struct MessageBus {
    handlers: HashMap<String, HashMap<TypeId, Box<dyn Fn(Box<dyn Any>)>>>,
}

impl MessageBus {
    fn new() -> SharedMessageBus {
        Rc::new(RefCell::new(Self {
            handlers: HashMap::new(),
        }))
    }

    fn register<M: 'static>(&mut self, topic: &str, handler: Box<dyn Fn(Box<dyn Any>)>) {
        self.handlers
            .entry(topic.to_string())
            .or_default()
            .insert(TypeId::of::<M>(), handler);
    }

    fn send<M: 'static>(&mut self, topic: &str, msg: M) {
        if let Some(handler) = self
            .handlers
            .get(topic)
            .and_then(|h| h.get(&TypeId::of::<M>()))
        {
            handler(Box::new(msg));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_handlers() {
        let msgbus = MessageBus::new();
        let actor = MyActor::new(msgbus.clone());

        msgbus.borrow_mut().register::<String>(
            "string_topic",
            Box::new(move |msg| {
                if let Some(msg) = msg.downcast_ref::<String>() {
                    actor.borrow_mut().handle(msg.clone());
                }
            }),
        );

        msgbus
            .borrow_mut()
            .send("string_topic", "start".to_string());

        msgbus.borrow_mut().send("bool_topic", true);
        msgbus.borrow_mut().send("bool_topic", false);
        msgbus.borrow_mut().send("bool_topic", true);

        assert_eq!(actor.borrow().true_count, 2);

        msgbus.borrow_mut().send("string_topic", "stop".to_string());

        msgbus.borrow_mut().send("bool_topic", true);
        assert_eq!(actor.borrow().true_count, 2);
    }
}
