from experiments.data.rust.core cimport trade_id_new
from experiments.data.rust.core cimport trade_tick_new
from experiments.data.rust.core cimport trade_tick_eq
from experiments.data.string cimport ustr_to_pystr
from experiments.data.string cimport pystr_to_cstr


cdef class TradeId:
    def __init__(self, str value not None):
        self._mem = trade_id_new(pystr_to_cstr(value))

    def __getstate__(self):
        return self.to_str()

    def __setstate__(self, state):
        self._mem = trade_id_new(pystr_to_cstr(state))

    def __eq__(self, TradeId other not None) -> bool:
        return self._mem.value == other._mem.value

    cdef str to_str(self):
        return ustr_to_pystr(self._mem.value)

    @property
    def value(self) -> str:
        return self.to_str()

    @staticmethod
    cdef TradeId from_mem_c(TradeId_t mem):
        cdef TradeId trade_id = TradeId.__new__(TradeId)
        trade_id._mem = mem
        return trade_id


cdef class TradeTick:
    def __init__(
        self,
        TradeId trade_id not None,
        uint64_t ts_event,
        uint64_t ts_init,
    ):
        self._mem = trade_tick_new(
            trade_id._mem,
            ts_event,
            ts_init,
        )

    def __eq__(self, TradeTick other) -> bool:
        return trade_tick_eq(&self._mem, &other._mem)

    def __getstate__(self):
        return (
            self.trade_id.value,
            self.ts_event,
            self.ts_init,
        )

    def __setstate__(self, state):
        self._mem = trade_tick_new(
            trade_id_new(pystr_to_cstr(state[0])),
            state[1],
            state[2],
        )

    @property
    def trade_id(self) -> InstrumentId:
        return TradeId.from_mem_c(self._mem.trade_id)

    @property
    def ts_event(self) -> int:
        return self._mem.ts_event

    @property
    def ts_init(self) -> int:
        return self._mem.ts_init
