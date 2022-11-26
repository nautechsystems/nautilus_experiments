# distutils: language = c++

from libcpp.vector cimport vector
from cpython.object cimport PyObject
from libc.stdio cimport printf

from experiments.data.rust.core cimport symbol_new
from experiments.data.rust.core cimport symbol_free
from experiments.data.rust.core cimport symbol_cvec_test
from experiments.data.rust.core cimport symbol_vector_test
from experiments.data.rust.core cimport Symbol_t
from experiments.data.rust.core cimport CVec


cdef class Symbol:
    def __init__(self, str value not None):
        self._mem = symbol_new(<PyObject *>value)
        
    def __del__(self):
        if self._mem.value != NULL:
            symbol_free(self._mem)  # `self._mem` moved to Rust (then dropped)

cdef void test_cvec(list items):
    cdef vector[Symbol_t] vec
    cdef CVec data
    [vec.push_back(<Symbol_t>(<Symbol>item)._mem) for item in items]
    data.ptr = <void*>vec.data()
    data.len = len(items)
    data.cap = 0
    symbol_cvec_test(data)

cdef void test_vector(list items):
    cdef vector[Symbol_t] vec
    [vec.push_back(<Symbol_t>(<Symbol>item)._mem) for item in items]
    symbol_vector_test(<void*>vec.data(), len(items))
