from experiments.data.rust.core cimport Symbol_t
from experiments.data.rust.core cimport CVec

from libc.stdint cimport uint64_t

cdef class Symbol:
    cdef Symbol_t _mem
    cdef uint64_t val

    @staticmethod
    cdef Symbol from_mem(Symbol_t* mem)

    @staticmethod
    cdef void send_list(list items)
    
    @staticmethod
    cdef list receive_buffer()
