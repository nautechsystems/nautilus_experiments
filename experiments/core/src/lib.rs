use pyo3::types::PyString;
use pyo3::{ffi, FromPyPointer, Python};

/// Represents a single quote tick in a financial market.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QuoteTick {
    pub instrument_id: InstrumentId,
}

#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
#[allow(clippy::box_collection)] // C ABI compatibility
pub struct InstrumentId {
    pub symbol: Symbol,
}

#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
#[allow(clippy::box_collection)] // C ABI compatibility
pub struct Symbol {
    value: Box<String>,
}

impl Symbol {
    pub fn new(s: String) -> Symbol {
        Symbol {
            value: Box::new(s),
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
    let v = Python::with_gil(|py| PyString::from_borrowed_ptr(py, ptr).to_string());
    println!("{}", v);
    v
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
    QuoteTick { instrument_id }
}

#[no_mangle]
pub extern "C" fn quote_tick_print(tick: QuoteTick) -> QuoteTick {
    println!("{:?}", tick.instrument_id.symbol.value.as_bytes());
    println!("{}", tick.instrument_id.symbol.value);
    println!("{:?}", tick.instrument_id.symbol);
    println!("{:?}", tick.instrument_id);
    println!("{:?}", tick);
    tick
}

/// Returns a Nautilus identifier from a valid Python object pointer.
///
/// # Safety
/// - Assumes `ptr` is borrowed from a valid Python UTF-8 `str`.
#[no_mangle]
pub unsafe extern "C" fn symbol_new(ptr: *mut ffi::PyObject) -> Symbol {
    println!("symbol_new");
    println!("{}", (*ptr).ob_refcnt);
    println!("{:?}", (*ptr).ob_type);
    let value = pystr_to_string(ptr);
    println!("{}", value);
    Symbol::new(value)
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
    symbol: Symbol,
) -> InstrumentId {
    InstrumentId { symbol }
}

/// Frees the memory for the given `instrument_id` by dropping.
#[no_mangle]
pub extern "C" fn instrument_id_free(instrument_id: InstrumentId) {
    drop(instrument_id); // Memory freed here
}
