from experiments.data.rust.core cimport Symbol_t

from libc.stdint cimport uint64_t

cdef class Symbol:
    cdef Symbol_t _mem
    cdef uint64_t val
    
    @staticmethod
    cdef Symbol from_raw(Symbol_t mem)
    