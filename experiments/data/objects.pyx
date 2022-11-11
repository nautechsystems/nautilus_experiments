from cpython.object cimport PyObject
from libc.stdio cimport printf

from experiments.data.rust.core cimport symbol_copy
from experiments.data.rust.core cimport symbol_new
from experiments.data.rust.core cimport symbol_free


cdef class Symbol:
    def __cinit__(self, str value not None):
        printf("cython symbol cinit\n")
        self._mem = symbol_new(<PyObject *>value)
        printf("cinit self._mem  %p\n", self._mem.value);
        printf("%.5s\n", self._mem.value)
        self.val = 1
        
    def __init__(self, str value not None):
        print("symbol init")
        
    def __del__(self):
        print("del")
        print(self.val)
        printf("del self._mem %p\n", self._mem.value)
        # printf("%s\n", self._mem.value)
        printf("symbol del: don't free memory\n")

    def __dealloc__(self) -> None:
        print("dealloc")
        print(self.val)
        printf("symbol dealloc: free memory\n")
        printf("dealloc before free self._mem %p\n", self._mem.value);
        printf("%.5s\n", self._mem.value)
        symbol_free(self._mem)  # `self._mem` moved to Rust (then dropped)
        printf("dealloc after free self._mem %p\n", self._mem.value);
        printf("%.5s\n", self._mem.value)

    @staticmethod
    cdef Symbol from_raw(Symbol_t s):
        cdef Symbol symbol = Symbol.__new__(Symbol)
        symbol._mem = symbol_copy(&s)

        return symbol

    @staticmethod
    def from_raw_py(Symbol symbol) -> Symbol:
        return Symbol.from_raw(symbol._mem)
