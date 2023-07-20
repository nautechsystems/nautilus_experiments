use std::ffi::{c_char, CStr, CString};
use ustr::Ustr;
use std::fmt::{Display, Formatter};

#[repr(C)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct Symbol {
    value: Ustr,
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct Venue {
    value: Ustr,
}

impl Display for Venue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct InstrumentId {
    pub symbol: Symbol,
    pub venue: Venue,
}

impl InstrumentId {
    fn new(s: &str) -> Self {
        match s.rsplit_once('.') {
            Some((symbol, venue)) => Self {
                symbol: Symbol {
                    value: Ustr::from(symbol),
                },
                venue: Venue {
                    value: Ustr::from(venue),
                },
            },
            None => panic!("Cannot split"),
        }
    }
}

impl Display for InstrumentId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.symbol, self.venue)
    }
}

/// Represents a single quote tick in a financial market.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct QuoteTick {
    pub instrument_id: InstrumentId,
}

////////////////////////////////////////////////////////////////////////////////
// C API
////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "C" fn quote_tick_new(instrument_id: InstrumentId) -> QuoteTick {
    QuoteTick { instrument_id }
}

#[no_mangle]
pub extern "C" fn quote_tick_eq(lhs: &QuoteTick, rhs: &QuoteTick) -> u8 {
    u8::from(lhs == rhs)
}

#[no_mangle]
pub unsafe extern "C" fn instrument_id_new_from_parts(
    symbol: Symbol,
    venue: Venue,
) -> InstrumentId {
    InstrumentId { symbol, venue }
}

#[no_mangle]
pub unsafe extern "C" fn instrument_id_new_from_cstr(ptr: *const c_char) -> InstrumentId {
    let s = CStr::from_ptr(ptr).to_str().expect("CStr::from_ptr failed");
    InstrumentId::new(s)
}

#[no_mangle]
pub extern "C" fn instrument_id_to_cstr(instrument_id: InstrumentId) -> *const c_char {
    CString::new(instrument_id.to_string().as_str())
        .expect("failed")
        .into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn symbol_new_from_cstr(ptr: *const c_char) -> Symbol {
    Symbol {
        value: Ustr::from(CStr::from_ptr(ptr).to_str().expect("CStr::from_ptr failed")),
    }
}

#[no_mangle]
pub unsafe extern "C" fn venue_new_from_cstr(ptr: *const c_char) -> Venue {
    Venue {
        value: Ustr::from(CStr::from_ptr(ptr).to_str().expect("CStr::from_ptr failed")),
    }
}
