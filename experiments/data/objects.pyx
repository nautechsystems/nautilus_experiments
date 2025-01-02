from experiments.data.rust.core cimport trade_tick_new
from experiments.data.rust.core cimport trade_tick_eq
from experiments.data.rust.core cimport uint128_t
from experiments.data.rust.core cimport int128_t

cdef class TradeTick:
    def __init__(
        self,
        uint128_t ts_event,
        int128_t ts_init,
    ):
        self._mem = trade_tick_new(ts_event, ts_init)

    def __eq__(self, TradeTick other) -> bool:
        return trade_tick_eq(&self._mem, &other._mem)

    def __getstate__(self):
        return (self.ts_event, self.ts_init)

    def __setstate__(self, state):
        self._mem = trade_tick_new(state[0], state[1])

    @property
    def ts_event(self) -> int:
        return self._mem.ts_event

    @property
    def ts_init(self) -> int:
        return self._mem.ts_init
