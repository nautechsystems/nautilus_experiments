from experiments.data.rust.core cimport symbol_new_from_cstr
from experiments.data.rust.core cimport venue_new_from_cstr
from experiments.data.rust.core cimport instrument_id_new_from_cstr
from experiments.data.rust.core cimport instrument_id_new_from_parts
from experiments.data.rust.core cimport instrument_id_to_cstr
from experiments.data.rust.core cimport quote_tick_new
from experiments.data.rust.core cimport quote_tick_eq
from experiments.data.string cimport cstr_to_pystr
from experiments.data.string cimport pystr_to_cstr


cdef class Symbol:
    def __init__(self, str value not None):
        self._mem = symbol_new_from_cstr(pystr_to_cstr(value))
        
    def __str__(self) -> str:
        return self.to_str()

    def __eq__(self, Symbol other) -> bool:
        return self._mem.value == other._mem.value

    def __repr__(self) -> str:
        return f"{type(self).__name__}('{self.to_str()}')"

    def __getstate__(self):
        return self.to_str()
        
    def __setstate__(self, state):
        self._mem = symbol_new_from_cstr(pystr_to_cstr(state))

    cdef str to_str(self):
        return cstr_to_pystr(self._mem.value)

    @property
    def value(self) -> str:
        return self.to_str()


cdef class Venue:
    def __init__(self, str value not None):
        self._mem = venue_new_from_cstr(pystr_to_cstr(value))
        
    def __str__(self) -> str:
        return self.to_str()

    def __eq__(self, Venue other) -> bool:
        return self._mem.value == other._mem.value

    def __repr__(self) -> str:
        return f"{type(self).__name__}('{self.to_str()}')"

    def __getstate__(self):
        return self.to_str()
        
    def __setstate__(self, state):
        self._mem = venue_new_from_cstr(pystr_to_cstr(state))

    cdef str to_str(self):
        return cstr_to_pystr(self._mem.value)

    @property
    def value(self) -> str:
        return self.to_str()


cdef class InstrumentId:
    def __init__(self, Symbol symbol not None, Venue venue not None):
        self._mem = instrument_id_new_from_parts(symbol._mem, venue._mem)

    def __getstate__(self):
        return self.to_str()

    def __setstate__(self, state):
        self._mem = instrument_id_new_from_cstr(pystr_to_cstr(state))

    def __eq__(self, InstrumentId other not None) -> bool:
        return self._mem.symbol.value == other._mem.symbol.value and self._mem.venue.value == other._mem.venue.value

    cdef str to_str(self):
        return cstr_to_pystr(instrument_id_to_cstr(self._mem))

    @property
    def value(self) -> str:
        return self.to_str()

    @staticmethod
    def from_string(symbol_str):
        return InstrumentId.from_str(symbol_str)

    @staticmethod
    cdef InstrumentId from_str(str value):
        cdef InstrumentId instrument_id = InstrumentId.__new__(InstrumentId)
        instrument_id._mem = instrument_id_new_from_cstr(pystr_to_cstr(value))
        return instrument_id

    @staticmethod
    cdef InstrumentId from_mem_c(InstrumentId_t mem):
        cdef InstrumentId instrument_id = InstrumentId.__new__(InstrumentId)
        instrument_id._mem = mem
        return instrument_id

cdef class QuoteTick:
    def __init__(self, InstrumentId instrument_id not None):
        self._mem = quote_tick_new(instrument_id._mem)
        self.instrument_id 

    def __getstate__(self):
        return self.instrument_id.value

    def __setstate__(self, state):
        self._mem = quote_tick_new(instrument_id_new_from_cstr(pystr_to_cstr(state)))

    def __eq__(self, QuoteTick other) -> bool:
        return quote_tick_eq(&self._mem, &other._mem)

    @property
    def instrument_id(self) -> InstrumentId:
        return InstrumentId.from_mem_c(self._mem.instrument_id)
