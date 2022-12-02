# distutils: language = c++

from libcpp.vector cimport vector
from cpython.object cimport PyObject
from libc.stdio cimport printf

from experiments.data.rust.core cimport symbol_new
from experiments.data.rust.core cimport symbol_free
from experiments.data.rust.core cimport symbol_vec_text
from experiments.data.rust.core cimport Symbol_t

from experiments.data.rust.core cimport instrument_id_clone
from experiments.data.rust.core cimport instrument_id_free
from experiments.data.rust.core cimport instrument_id_new
from experiments.data.rust.core cimport quote_tick_free
from experiments.data.rust.core cimport quote_tick_new
from experiments.data.rust.core cimport quote_tick_debug

cdef class Symbol:
    def __init__(self, str value not None):
        self._mem = symbol_new(<PyObject *>value)
        
    def __del__(self):
        if self._mem.value != NULL:
            symbol_free(self._mem)  # `self._mem` moved to Rust (then dropped)

cdef void* create_vector(list items):
    cdef vector[Symbol_t] vec
    [vec.push_back(<Symbol_t>(<Symbol>item)._mem) for item in items]
    symbol_vec_text(<void*>vec.data(), len(items))

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
        self._mem = instrument_id_new(<PyObject *>symbol)
        self.symbol = symbol

    def __del__(self):
        if self._mem.symbol.value != NULL:
            instrument_id_free(self._mem)
