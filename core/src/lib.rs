use pyo3::types::PyString;
use pyo3::{ffi, FromPyPointer, Python};

/// Represents a single quote tick in a financial market.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QuoteTick {
    pub instrument_id: InstrumentId,
}

impl QuoteTick {
    pub fn new(instrument_id: InstrumentId) -> QuoteTick {
        QuoteTick { instrument_id }
    }
}

#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
#[allow(clippy::box_collection)] // C ABI compatibility
pub struct InstrumentId {
    pub symbol: Symbol,
    pub venue: Symbol,
}

impl From<&str> for InstrumentId {
    fn from(s: &str) -> Self {
        let pieces: Vec<&str> = s.split('.').collect();
        InstrumentId {
            symbol: Symbol::new(pieces[0]),
            venue: Symbol::new(pieces[1]),
        }
    }
}

impl From<&String> for InstrumentId {
    fn from(s: &String) -> Self {
        let pieces: Vec<&str> = s.split('.').collect();
        InstrumentId {
            symbol: Symbol::new(pieces[0]),
            venue: Symbol::new(pieces[1]),
        }
    }
}

impl InstrumentId {
    pub fn new(symbol: Symbol, venue: Symbol) -> InstrumentId {
        InstrumentId { symbol, venue }
    }
}

#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
#[allow(clippy::box_collection)] // C ABI compatibility
pub struct Symbol {
    value: Box<String>,
}

impl Symbol {
    pub fn new(s: &str) -> Symbol {
        Symbol {
            value: Box::new(s.to_string()),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Pyo3 API
////////////////////////////////////////////////////////////////////////////////

/// Returns an owned string from a valid Python object pointer.
///
/// # Safety
/// - Panics if `ptr` is null.
/// - Assumes `ptr` is borrowed from a valid Python UTF-8 `str`.
#[inline(always)]
pub unsafe fn pystr_to_string(ptr: *mut ffi::PyObject) -> String {
    assert!(!ptr.is_null(), "pointer was NULL");
    Python::with_gil(|py| PyString::from_borrowed_ptr(py, ptr).to_string())
}

////////////////////////////////////////////////////////////////////////////////
// C API
////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "C" fn quote_tick_free(tick: QuoteTick) {
    drop(tick); // Memory freed here
}

#[no_mangle]
pub extern "C" fn quote_tick_from_raw(instrument_id: InstrumentId) -> QuoteTick {
    QuoteTick::new(instrument_id)
}

/// Returns a Nautilus identifier from a valid Python object pointer.
///
/// # Safety
/// - Assumes `ptr` is borrowed from a valid Python UTF-8 `str`.
#[no_mangle]
pub unsafe extern "C" fn symbol_new(ptr: *mut ffi::PyObject) -> Symbol {
    Symbol::new(pystr_to_string(ptr).as_str())
}

/// Frees the memory for the given `symbol` by dropping.
#[no_mangle]
pub extern "C" fn symbol_free(symbol: Symbol) {
    drop(symbol); // Memory freed here
}

/// Returns a Nautilus identifier from valid Python object pointers.
///
/// # Safety
/// - Assumes `symbol_ptr` is borrowed from a valid Python UTF-8 `str`.
/// - Assumes `venue_ptr` is borrowed from a valid Python UTF-8 `str`.
#[no_mangle]
pub unsafe extern "C" fn instrument_id_new(
    symbol_ptr: *mut ffi::PyObject,
    venue_ptr: *mut ffi::PyObject,
) -> InstrumentId {
    let symbol = symbol_new(symbol_ptr);
    let venue = symbol_new(venue_ptr);
    InstrumentId::new(symbol, venue)
}

/// Frees the memory for the given `instrument_id` by dropping.
#[no_mangle]
pub extern "C" fn instrument_id_free(instrument_id: InstrumentId) {
    drop(instrument_id); // Memory freed here
}
