from experiments.data.rust.core cimport Symbol_t
from experiments.data.rust.core cimport CVec

from libc.stdint cimport uint64_t

cdef class Symbol:
    cdef Symbol_t _mem
    cdef uint64_t val

    @staticmethod
    cdef inline Symbol from_mem_void(Symbol_t* mem)

    @staticmethod
    cdef inline Symbol from_mem(Symbol_t* mem)

cdef void send_list(list items)

cdef list receive_buffer(CVec buffer)
