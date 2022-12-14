mod string;

use std::ffi::{c_char, c_void, CStr};
use std::rc::Rc;
use std::slice;

use pyo3::types::PyString;
use pyo3::{ffi, FromPyPointer, Python};
use crate::string::string_to_cstr;

#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
#[allow(clippy::box_collection)] // C ABI compatibility
pub struct Symbol {
    value: Box<Rc<String>>,
}

#[repr(C)]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct InstrumentId {
    pub symbol: Symbol,
}
/// Represents a single quote tick in a financial market.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
pub unsafe extern "C" fn instrument_id_new_from_pystr(ptr: *mut ffi::PyObject) -> InstrumentId {
    let symbol = symbol_new(ptr);
    InstrumentId { symbol }
}

#[no_mangle]
pub unsafe extern "C" fn instrument_id_new(symbol: &Symbol) -> InstrumentId {
    InstrumentId { symbol: symbol.clone() }
}

#[no_mangle]
pub unsafe extern "C" fn symbol_new(ptr: *mut ffi::PyObject) -> Symbol {
    let v = Python::with_gil(|py| PyString::from_borrowed_ptr(py, ptr).to_string());
    Symbol {
        value: Box::new(Rc::new(v)),
    }
}

#[no_mangle]
pub unsafe extern "C" fn symbol_new_from_cstr(ptr: *const c_char) -> Symbol {
    Symbol {
        value: Box::new(Rc::new(CStr::from_ptr(ptr).to_str().expect("CStr::from_ptr failed").to_string()))
    }
}

/// Returns a [Symbol] as a C string pointer.
#[no_mangle]
pub extern "C" fn symbol_to_cstr(symbol: &Symbol) -> *const c_char {
    string_to_cstr(symbol.value.as_str())
}

#[no_mangle]
pub extern "C" fn quote_tick_clone(tick: &QuoteTick) -> QuoteTick {
    tick.clone()
}

#[no_mangle]
pub extern "C" fn instrument_id_clone(instrument_id: &InstrumentId) -> InstrumentId {
    instrument_id.clone()
}

/// Frees the memory for the given `symbol` by dropping.
#[no_mangle]
pub extern "C" fn quote_tick_free(tick: QuoteTick) {
    drop(tick); // Memory freed here
}

/// Frees the memory for the given `symbol` by dropping.
#[no_mangle]
pub extern "C" fn instrument_id_free(instrument_id: InstrumentId) {
    drop(instrument_id); // Memory freed here
}

/// Frees the memory for the given `symbol` by dropping.
#[no_mangle]
pub extern "C" fn symbol_free(symbol: Symbol) {
    drop(symbol); // Memory freed here
}

#[no_mangle]
pub extern "C" fn instrument_id_debug(instrument_id: &InstrumentId) {
    dbg!(&instrument_id.symbol.value);
    dbg!(Rc::strong_count(&instrument_id.symbol.value));
}

#[no_mangle]
pub extern "C" fn quote_tick_debug(tick: &QuoteTick) {
    dbg!(&tick.instrument_id.symbol.value);
    dbg!(Rc::strong_count(&tick.instrument_id.symbol.value));
}

#[no_mangle]
pub extern "C" fn symbol_debug(symbol: &Symbol) {
    dbg!(&symbol.value);
    dbg!(Rc::strong_count(&symbol.value));
}

#[no_mangle]
pub extern "C" fn symbol_vec_text(data: *mut c_void, len: usize) {
    let data: &[Symbol] = unsafe { slice::from_raw_parts(data as *const Symbol, len) };
    let v = &data[len - 1];
    dbg!(Rc::strong_count(&v.value));
    dbg!(len, &data[len - 1]);
}
