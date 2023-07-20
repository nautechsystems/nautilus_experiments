from experiments.data.rust.core cimport TradeId_t
from experiments.data.rust.core cimport TradeTick_t

from libc.stdint cimport uint64_t, uint8_t

cdef class TradeId:
    cdef TradeId_t _mem
    cdef str to_str(self)

    @staticmethod
    cdef TradeId from_mem_c(TradeId_t mem)

cdef class TradeTick:
    cdef TradeTick_t _mem
