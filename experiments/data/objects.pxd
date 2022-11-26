from experiments.data.rust.core cimport Symbol_t

from libc.stdint cimport uint64_t

cdef class Symbol:
    cdef Symbol_t _mem
    cdef uint64_t val

cdef void test_cvec(list items)

cdef void test_vector(list items)
