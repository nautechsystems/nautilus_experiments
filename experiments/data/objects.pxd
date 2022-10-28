from experiments.data.rust.core cimport Symbol_t
from experiments.data.rust.core cimport InstrumentId_t
from experiments.data.rust.core cimport QuoteTick_t


cdef class Symbol:
    cdef Symbol_t _mem

cdef class InstrumentId:
    cdef InstrumentId_t _mem

    cdef readonly Symbol symbol

cdef class QuoteTick:
    cdef QuoteTick_t _mem
