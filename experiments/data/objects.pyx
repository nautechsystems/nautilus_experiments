from cpython.object cimport PyObject
from libc.stdio cimport printf

from experiments.data.rust.core cimport quote_tick_free
from experiments.data.rust.core cimport quote_tick_from_raw
from experiments.data.rust.core cimport quote_tick_print
from experiments.data.rust.core cimport symbol_new
from experiments.data.rust.core cimport symbol_free
from experiments.data.rust.core cimport instrument_id_new
from experiments.data.rust.core cimport instrument_id_free


cdef class Symbol:
    def __init__(self, str value):
        self._mem = symbol_new(<PyObject *>value)

    def __dealloc__(self) -> None:
        printf("symbol dealloc\n")
        symbol_free(self._mem)  # `self._mem` moved to Rust (then dropped)


cdef class InstrumentId:
    def __init__(self, Symbol symbol not None):

        self._mem = instrument_id_new(
            symbol._mem
        )
        self.symbol = symbol

    def __dealloc__(self) -> None:
        printf("instrument dealloc\n")
        instrument_id_free(self._mem)  # `self._mem` moved to Rust (then dropped)

cdef class QuoteTick:

    def __init__(
        self,
        InstrumentId instrument_id not None,
    ):

        self._mem = quote_tick_from_raw(instrument_id._mem)

    def __dealloc__(self) -> None:
        printf("tick dealloc\n")
        quote_tick_free(self._mem)  # `self._mem` moved to Rust (then dropped)
        
    def print(self) -> None:
        self._mem = quote_tick_print(self._mem)
