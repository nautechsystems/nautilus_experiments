pub struct OrderInitialized {
    pub trader_id: String,
    pub strategy_id: String,
    pub ts_event: u64,
    pub ts_init: u64,
    pub trailing_offset: Option<u64>,
    pub trailing_offset_type: Option<u64>,
}

pub struct OrderExpired {
    pub trader_id: String,
    pub strategy_id: String,
    pub ts_event: u64,
    pub ts_init: u64,
    pub reconcilliation: u64,
}
