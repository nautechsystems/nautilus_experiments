use std::ops::{Deref, DerefMut};

#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Order {
    pub trader_id: String,
    pub strategy_id: String,
    pub ts_event: u64,
    pub ts_init: u64,
}

#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct OrderInitialized {
    pub order: Order,
    pub trailing_offset: Option<u64>,
    pub trailing_offset_type: Option<u64>,
}

impl Deref for OrderInitialized {
    type Target = Order;

    fn deref(&self) -> &Self::Target {
        &self.order
    }
}

impl DerefMut for OrderInitialized {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.order
    }
}

#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct OrderExpired {
    pub order: Order,
    pub reconcilliation: u64,
}

impl Deref for OrderExpired {
    type Target = Order;

    fn deref(&self) -> &Self::Target {
        &self.order
    }
}

impl DerefMut for OrderExpired {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.order
    }
}



mod tests {
    use crate::{Order, OrderExpired, OrderInitialized};

    #[test]
    fn build_expired_order() {
        let mut expired = OrderExpired {
            order: Order {
                trader_id: "hi".to_string(),
                strategy_id: "bye".to_string(),
                ts_event: 1,
                ts_init: 1,
            },
            reconcilliation: 2,
        };

        assert_eq!(expired.ts_event, 1);
        assert_eq!(expired.trader_id, "hi");
        assert_eq!(expired.reconcilliation, 2);
        
        // mutate through order attribute only
        expired.trader_id = "oh no".to_string();
        expired.reconcilliation = 3;
        
        assert_eq!(expired.trader_id, "oh no");
        assert_eq!(expired.reconcilliation, 3);
    }

    #[test]
    fn build_initialized_order() {
        let mut init = OrderInitialized {
            order: Order {
                trader_id: "hi".to_string(),
                strategy_id: "bye".to_string(),
                ts_event: 1,
                ts_init: 1,
            },
            trailing_offset: Some(2),
            trailing_offset_type: Some(3),
        };

        assert_eq!(init.ts_event, 1);
        assert_eq!(init.trader_id, "hi");
        assert_eq!(init.trailing_offset, Some(2));

        // mutate through order attribute only
        init.trader_id = "oh no".to_string();
        init.trailing_offset = Some(7);
        
        assert_eq!(init.trader_id, "oh no");
        assert_eq!(init.trailing_offset, Some(7));
    }
}
