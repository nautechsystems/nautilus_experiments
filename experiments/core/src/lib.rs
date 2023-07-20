use std::ffi::{c_char, CStr, CString};
use std::fmt::{Display, Formatter};
use ustr::Ustr;

#[repr(C)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct TradeId {
    value: Ustr,
}

impl Display for TradeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Represents a single quote tick in a financial market.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TradeTick {
    pub trade_id: TradeId,
    pub ts_event: u64,
    pub ts_init: u64,
}

////////////////////////////////////////////////////////////////////////////////
// C API
////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "C" fn trade_tick_new(trade_id: TradeId, ts_event: u64, ts_init: u64) -> TradeTick {
    TradeTick {
        trade_id,
        ts_event,
        ts_init,
    }
}

#[no_mangle]
pub extern "C" fn trade_tick_eq(lhs: &TradeTick, rhs: &TradeTick) -> u8 {
    assert_eq!(lhs.ts_event, rhs.ts_event);
    assert_eq!(lhs.ts_init, rhs.ts_init);
    assert_eq!(lhs.trade_id, rhs.trade_id);
    u8::from(lhs == rhs)
}

#[no_mangle]
pub unsafe extern "C" fn trade_id_new(ptr: *const c_char) -> TradeId {
    TradeId {
        value: Ustr::from(CStr::from_ptr(ptr).to_str().expect("CStr::from_ptr failed")),
    }
}
