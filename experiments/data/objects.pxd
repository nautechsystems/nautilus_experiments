from experiments.data.rust.core cimport TradeTick_t

from libc.stdint cimport uint64_t, uint8_t

cdef class TradeTick:
    cdef TradeTick_t _mem
