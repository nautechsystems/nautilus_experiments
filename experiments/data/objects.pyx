from cpython.object cimport PyObject
from libc.stdio cimport printf

from experiments.data.rust.core cimport symbol_new_from_cstr
from experiments.data.rust.core cimport symbol_free
from experiments.data.rust.core cimport symbol_to_cstr
from experiments.data.rust.core cimport symbol_debug

from experiments.data.rust.core cimport instrument_id_clone
from experiments.data.rust.core cimport instrument_id_free
from experiments.data.rust.core cimport instrument_id_new
from experiments.data.rust.core cimport instrument_id_new_from_pystr
from experiments.data.rust.core cimport quote_tick_free
from experiments.data.rust.core cimport quote_tick_new
from experiments.data.rust.core cimport quote_tick_debug
from experiments.data.rust.core cimport instrument_id_debug
from experiments.data.string cimport cstr_to_pystr
from experiments.data.string cimport pystr_to_cstr


cdef class Symbol:
    def __init__(self, str value not None):
        self._mem = symbol_new_from_cstr(pystr_to_cstr(value))
        
    def __del__(self):
        if self._mem.value != NULL:
            symbol_free(self._mem)  # `self._mem` moved to Rust (then dropped)

    def __str__(self) -> str:
        return self.to_str()

    def __repr__(self) -> str:
        return f"{type(self).__name__}('{self.to_str()}')"

    cdef str to_str(self):
        return cstr_to_pystr(symbol_to_cstr(&self._mem))

    def debug(self):
        symbol_debug(&self._mem)


cdef class QuoteTick:
    def __init__(self, InstrumentId instrument_id not None):
        self._mem = quote_tick_new(instrument_id_clone(&instrument_id._mem))

    def __del__(self):
        if self._mem.instrument_id.symbol.value != NULL:
            quote_tick_free(self._mem)

    def debug(self):
        quote_tick_debug(&self._mem)


cdef class InstrumentId:
    def __init__(self, Symbol symbol not None):
        self._mem = instrument_id_new(&symbol._mem)
        self.symbol = symbol

    def __del__(self):
        if self._mem.symbol.value != NULL:
            instrument_id_free(self._mem)

    def debug(self):
        instrument_id_debug(&self._mem)
        
    @staticmethod
    def from_string(symbol_str):
        return InstrumentId.from_str(symbol_str)

    @staticmethod
    cdef InstrumentId from_str(str value):
        cdef InstrumentId instrument_id = InstrumentId.__new__(InstrumentId)
        instrument_id._mem = instrument_id_new_from_pystr(
            <PyObject *>value,
        )
        instrument_id.symbol = Symbol(value)

        return instrument_id
