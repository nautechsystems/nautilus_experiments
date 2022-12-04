from cpython.object cimport PyObject

from experiments.data.rust.core cimport uuid4_free
from experiments.data.rust.core cimport uuid4_from_pystr
from experiments.data.rust.core cimport uuid4_new
from experiments.data.rust.core cimport uuid4_to_pystr

cdef class UUID4:
    def __init__(self, str value = None):
        if value is None:
            # Create a new UUID4 from Rust
            self._mem = uuid4_new()  # `UUID4_t` owned from Rust
        else:
            # `value` borrowed by Rust, `UUID4_t` owned from Rust
            self._mem = uuid4_from_pystr(<PyObject *>value)

    cdef str to_str(self):
        return <str>uuid4_to_pystr(&self._mem)

    def __del__(self) -> None:
        if self._mem.value != NULL:
            uuid4_free(self._mem)  # `self._uuid4` moved to Rust (then dropped)

    def __getstate__(self):
        return self.to_str()

    def __setstate__(self, state):
        self._mem = uuid4_from_pystr(<PyObject *>state)

    def __eq__(self, UUID4 other) -> bool:
        return self.value == other.value

    def __hash__(self) -> int:
        return hash(self.value)

    def __str__(self) -> str:
        return self.to_str()

    def __repr__(self) -> str:
        return f"{type(self).__name__}('{self.value}')"
