from cpython.object cimport PyObject

from experiments.data.rust.core cimport uuid4_free
from experiments.data.rust.core cimport uuid4_from_pystr
from experiments.data.rust.core cimport uuid4_new
from experiments.data.rust.core cimport uuid4_to_pystr

cdef class UUID4:
    def __init__(self, str value=None):
        if value is None:
            # Create a new UUID4 from Rust
            self._uuid4 = uuid4_new()  # `UUID4_t` owned from Rust
            self.value = <str>uuid4_to_pystr(&self._uuid4)  # `PyUnicode` owned from Rust
        else:
            self._uuid4 = self._uuid4_from_pystr(value)
            self.value = value

    cdef UUID4_t _uuid4_from_pystr(self, str value) except *:
        return uuid4_from_pystr(<PyObject *>value)  # `value` borrowed by Rust, `UUID4_t` owned from Rust

    def __del__(self) -> None:
        uuid4_free(self._uuid4)  # `self._uuid4` moved to Rust (then dropped)

    def __getstate__(self):
        return self.value

    def __setstate__(self, state):
        self._uuid4 = self._uuid4_from_pystr(state)
        self.value = state

    def __eq__(self, UUID4 other) -> bool:
        return self.value == other.value

    def __hash__(self) -> int:
        return hash(self.value)

    def __str__(self) -> str:
        return self.value

    def __repr__(self) -> str:
        return f"{type(self).__name__}('{self.value}')"
