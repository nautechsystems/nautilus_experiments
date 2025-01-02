/// Represents a single quote tick in a financial market.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TradeTick {
    pub ts_event: u128,
    pub ts_init: i128,
}

////////////////////////////////////////////////////////////////////////////////
// C API
////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn trade_tick_new(ts_event: u128, ts_init: i128) -> TradeTick {
    TradeTick { ts_event, ts_init }
}

#[no_mangle]
pub extern "C" fn trade_tick_eq(lhs: &TradeTick, rhs: &TradeTick) -> u8 {
    assert_eq!(lhs.ts_event, rhs.ts_event);
    assert_eq!(lhs.ts_init, rhs.ts_init);
    u8::from(lhs == rhs)
}
