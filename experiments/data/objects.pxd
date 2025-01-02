from experiments.data.rust.core cimport TradeTick_t

cdef class TradeTick:
    cdef TradeTick_t _mem
