from experiments.data.rust.core cimport TradeTick_t
from experiments.data.rust.core cimport MAX

cdef class TradeTick:
    cdef TradeTick_t _mem
