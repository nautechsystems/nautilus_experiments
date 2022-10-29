from experiments.data.rust.core cimport Symbol_t


cdef class Symbol:
    cdef Symbol_t _mem
    
    @staticmethod
    cdef Symbol from_raw(Symbol_t mem)
