# distutils: language = c++

from libcpp.vector cimport vector
from cpython.object cimport PyObject
from libc.stdio cimport printf

from experiments.data.rust.core cimport symbol_new
from experiments.data.rust.core cimport symbol_free
from experiments.data.rust.core cimport symbol_vec_test
from experiments.data.rust.core cimport Symbol_t


cdef class Symbol:
    def __init__(self, str value not None):
        self._mem = symbol_new(<PyObject *>value)
        
    def __del__(self):
        if self._mem.value != NULL:
            symbol_free(self._mem)  # `self._mem` moved to Rust (then dropped)

cdef void* create_vector(list items):
    cdef vector[Symbol_t] vec
    [vec.push_back(<Symbol_t>(<Symbol>item)._mem) for item in items]
    symbol_vec_test(<void*>vec.data(), len(items))
