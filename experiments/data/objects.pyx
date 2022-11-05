from cpython.object cimport PyObject
from libc.stdio cimport printf

from experiments.data.rust.core cimport symbol_copy
from experiments.data.rust.core cimport symbol_new
from experiments.data.rust.core cimport symbol_free


cdef class Symbol:
    def __init__(self, str value not None):
        self._mem = symbol_new(<PyObject *>value)

    def __del__(self):
        printf("symbol del: don't free memory\n")

    def __dealloc__(self) -> None:
        printf("symbol dealloc: free memory\n")
        symbol_free(self._mem)  # `self._mem` moved to Rust (then dropped)

    @staticmethod
    cdef Symbol from_raw(Symbol_t s):
        cdef Symbol symbol = Symbol.__new__(Symbol)
        symbol._mem = symbol_copy(&s)

        return symbol

    @staticmethod
    def from_raw_py(Symbol symbol) -> Symbol:
        return Symbol.from_raw(symbol._mem)
