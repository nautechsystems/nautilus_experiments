use derive_builder::{self, Builder};
use std::ops::{Deref, DerefMut};

#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq, Debug, Builder)]
pub struct Order {
    pub trader_id: String,
    pub strategy_id: String,
    pub ts_event: u64,
    pub ts_init: u64,
}

macro_rules! impl_derefs_for_order {
    ($struct: ident) => {
        impl Deref for $struct {
            type Target = Order;

            fn deref(&self) -> &Self::Target {
                &self.order
            }
        }

        impl DerefMut for $struct {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.order
            }
        }
    };
}

#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq, Debug, Builder)]
pub struct OrderInitialized {
    pub order: Order,
    pub trailing_offset: Option<u64>,
    pub trailing_offset_type: Option<u64>,
}

#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq, Debug, Builder)]
pub struct OrderExpired {
    pub order: Order,
    pub reconcilliation: u64,
}

impl_derefs_for_order!(OrderInitialized);
impl_derefs_for_order!(OrderExpired);

mod tests {
    use crate::{
        Order, OrderBuilder, OrderExpired, OrderExpiredBuilder, OrderInitialized,
        OrderInitializedBuilder,
    };

    #[test]
    fn build_expired_order() {
        let order = Order {
            trader_id: "hi".to_string(),
            strategy_id: "bye".to_string(),
            ts_event: 1,
            ts_init: 1,
        };
        let mut builder = OrderExpiredBuilder::default();
        builder.order(order).reconcilliation(2);
        let expired = builder.build().unwrap();

        assert_eq!(expired.ts_event, 1);
        assert_eq!(expired.trader_id, "hi");
        assert_eq!(expired.reconcilliation, 2);

        // mutate through order attribute only
        builder.reconcilliation(3);
        let mut expired = builder.build().unwrap();
        expired.trader_id = "oh no".to_string();

        assert_eq!(expired.trader_id, "oh no");
        assert_eq!(expired.reconcilliation, 3);
    }

    #[test]
    fn build_initialized_order() {
        let order = Order {
            trader_id: "hi".to_string(),
            strategy_id: "bye".to_string(),
            ts_event: 1,
            ts_init: 1,
        };
        let mut builder = OrderInitializedBuilder::default();
        builder
            .order(order)
            .trailing_offset(Some(2))
            .trailing_offset_type(Some(3));
        let init = builder.build().unwrap();

        assert_eq!(init.ts_event, 1);
        assert_eq!(init.trader_id, "hi");
        assert_eq!(init.trailing_offset, Some(2));

        // mutate through order attribute only
        builder.trailing_offset(Some(7));
        let mut init = builder.build().unwrap();
        init.trader_id = "oh no".to_string();

        assert_eq!(init.trader_id, "oh no");
        assert_eq!(init.trailing_offset, Some(7));
    }
}
