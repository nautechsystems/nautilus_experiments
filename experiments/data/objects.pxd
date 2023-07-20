from experiments.data.rust.core cimport Venue_t
from experiments.data.rust.core cimport Symbol_t
from experiments.data.rust.core cimport QuoteTick_t
from experiments.data.rust.core cimport InstrumentId_t


cdef class Symbol:
    cdef Symbol_t _mem
    cdef str to_str(self)

cdef class Venue:
    cdef Venue_t _mem
    cdef str to_str(self)


cdef class InstrumentId:
    cdef InstrumentId_t _mem
    cdef str to_str(self)
    @staticmethod
    cdef InstrumentId from_str(str value)
    @staticmethod
    cdef InstrumentId from_mem_c(InstrumentId_t mem)

cdef class QuoteTick:
    cdef QuoteTick_t _mem
