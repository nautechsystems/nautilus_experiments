from experiments.data.rust.core cimport Symbol_t

from libc.stdint cimport uint64_t

cdef class Symbol:
    cdef Symbol_t _mem
    cdef uint64_t val

cdef void* create_vector(list items)
