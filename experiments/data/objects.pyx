from cpython.object cimport PyObject
from libc.stdio cimport printf

from cpython.mem cimport PyMem_Free
from cpython.mem cimport PyMem_Malloc
from cpython.mem cimport PyMem_Realloc

from experiments.data.rust.core cimport symbol_new
from experiments.data.rust.core cimport symbol_free
from experiments.data.rust.core cimport symbol_clone_void
from experiments.data.rust.core cimport symbol_clone
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

    @staticmethod
    cdef inline Symbol from_mem_void(Symbol_t* mem):
        cdef Symbol obj = Symbol.__new__(Symbol)
        obj._mem = symbol_clone_void(<void*>mem)
        return obj

    @staticmethod
    cdef inline Symbol from_mem(Symbol_t* mem):
        cdef Symbol obj = Symbol.__new__(Symbol)
        obj._mem = symbol_clone(mem)
        return obj

cdef void send_list(list items):
    cdef Symbol_t* data
    data = <Symbol_t*> PyMem_Malloc(len(items) * sizeof(Symbol_t))
    if not data:
        raise MemoryError()
    for i in range(len(items)):
        data[i] = (<Symbol>items[i])._mem

    symbol_vector_test(<void*>data, len(items))

    PyMem_Free(data)

cdef list receive_buffer(CVec buffer):
    data = []
    for i in range(0, buffer.len):
        data.append(Symbol.from_mem(&(<Symbol_t*>buffer.ptr)[i]))
        
    return data
