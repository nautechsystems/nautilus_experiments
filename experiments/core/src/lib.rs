#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct OrderInitialized {
    pub trader_id: String,
    pub strategy_id: String,
    pub ts_event: u64,
    pub ts_init: u64,
    pub trailing_offset: Option<u64>,
    pub trailing_offset_type: Option<u64>,
}

#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct OrderExpired {
    pub trader_id: String,
    pub strategy_id: String,
    pub ts_event: u64,
    pub ts_init: u64,
    pub reconcilliation: u64,
}

mod tests {
    use crate::{OrderExpired, OrderInitialized};

    #[test]
    fn build_expired_order() {
        let mut expired = OrderExpired {
            trader_id: "hi".to_string(),
            strategy_id: "bye".to_string(),
            ts_event: 1,
            ts_init: 1,
            reconcilliation: 2,
        };
    }

    fn build_initialized_order() {
        let mut init = OrderInitialized {
            trader_id: "hi".to_string(),
            strategy_id: "bye".to_string(),
            ts_event: 1,
            ts_init: 1,
            trailing_offset: Some(2),
            trailing_offset_type: Some(3),
        };
    }
}