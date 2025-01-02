cdef extern from "core.h":
    ctypedef unsigned long long uint128_t

from experiments.data.rust.core cimport trade_tick_new
from experiments.data.rust.core cimport trade_tick_eq

cdef class TradeTick:
    def __init__(
        self,
        uint128_t ts_event,
    ):
        self._mem = trade_tick_new(ts_event)

    def __eq__(self, TradeTick other) -> bool:
        return trade_tick_eq(&self._mem, &other._mem)

    def __getstate__(self):
        return (self.ts_event,)

    def __setstate__(self, state):
        self._mem = trade_tick_new(state[0])

    @property
    def ts_event(self) -> int:
        return self._mem.ts_event
