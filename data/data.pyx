from cpython.object cimport PyObject

from rust.core cimport quote_tick_free
from rust.core cimport quote_tick_from_raw
from rust.core cimport symbol_new
from rust.core cimport symbol_free
from rust.core cimport instrument_id_new
from rust.core cimport instrument_id_free


cdef class Symbol:
    def __init__(self, str value):
        self._mem = symbol_new(<PyObject *>value)

    def __dealloc__(self) -> None:
        symbol_free(self._mem)  # `self._mem` moved to Rust (then dropped)

cdef class InstrumentId:
    def __init__(self, Symbol symbol not None, Symbol venue not None):

        self._mem = instrument_id_new(
            <PyObject *>symbol,
            <PyObject *>venue,
        )
        self.symbol = symbol
        self.venue = venue

    def __dealloc__(self) -> None:
        instrument_id_free(self._mem)  # `self._mem` moved to Rust (then dropped)

    @staticmethod
    cdef InstrumentId from_raw_c(InstrumentId_t raw):
        cdef Symbol symbol = Symbol.__new__(Symbol)
        symbol._mem = raw.symbol

        cdef Symbol venue = Symbol.__new__(Symbol)
        venue._mem = raw.venue

        cdef InstrumentId instrument_id = InstrumentId.__new__(InstrumentId)
        instrument_id._mem = raw
        instrument_id.symbol = symbol
        instrument_id.venue = venue

        return instrument_id

    @staticmethod
    cdef InstrumentId from_str_c(str value):
        cdef list pieces = value.rsplit('.', maxsplit=1)

        if len(pieces) != 2:
            raise ValueError(f"The InstrumentId string value was malformed, was {value}")

        cdef InstrumentId instrument_id = InstrumentId.__new__(InstrumentId)
        instrument_id._mem = instrument_id_new(
            <PyObject *>pieces[0],
            <PyObject *>pieces[1],
        )
        instrument_id.symbol = Symbol(pieces[0])
        instrument_id.venue = Symbol(pieces[1])

        return instrument_id

cdef class QuoteTick:

    def __init__(
        self,
        InstrumentId instrument_id not None,
    ):

        self._mem = quote_tick_from_raw(
            instrument_id._mem,
        )

    def __dealloc__(self) -> None:
        quote_tick_free(self._mem)  # `self._mem` moved to Rust (then dropped)
