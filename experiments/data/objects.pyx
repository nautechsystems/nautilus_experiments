from cpython.object cimport PyObject
from cpython.pycapsule cimport PyCapsule_GetPointer

from experiments.data.rust.core cimport UUID4_t
from experiments.data.rust.core cimport uuid4_clone
from experiments.data.rust.core cimport uuid4_eq
from experiments.data.rust.core cimport uuid4_free
from experiments.data.rust.core cimport uuid4_from_pystr
from experiments.data.rust.core cimport uuid4_hash
from experiments.data.rust.core cimport uuid4_new
from experiments.data.rust.core cimport uuid4_to_pystr
from experiments.data.rust.core cimport CVec
from experiments.data.rust.core cimport cvec_free


cdef class UUID4:
    """
    Represents a pseudo-random UUID (universally unique identifier)
    version 4 based on a 128-bit label as specified in RFC 4122.

    Parameters
    ----------
    value : str, optional
        The UUID value. If ``None`` then a value will be generated.

    Warnings
    --------
    - Panics at runtime if `value` is not ``None`` and not a valid UUID.

    References
    ----------
    https://en.wikipedia.org/wiki/Universally_unique_identifier
    """

    def __init__(self, str value = None):
        if value is None:
            # Create a new UUID4 from Rust
            self._mem = uuid4_new()  # `UUID4_t` owned from Rust
        else:
            # `value` borrowed by Rust, `UUID4_t` owned from Rust
            self._mem = uuid4_from_pystr(<PyObject *>value)

    cdef str to_str(self):
        return pyobj_to_str(uuid4_to_pystr(&self._mem))

    def __del__(self) -> None:
        if self._mem.value != NULL:
            uuid4_free(self._mem)  # `self._mem` moved to Rust (then dropped)

    def __getstate__(self):
        return self.to_str()

    def __setstate__(self, state):
        self._mem = uuid4_from_pystr(<PyObject *>state)

    def __eq__(self, UUID4 other) -> bool:
        return uuid4_eq(&self._mem, &other._mem)

    def __hash__(self) -> int:
        return uuid4_hash(&self._mem)

    def __str__(self) -> str:
        return self.to_str()

    def __repr__(self) -> str:
        return f"{type(self).__name__}('{self}')"

    @property
    def value(self) -> str:
        return self.to_str()
        
    def from_capsule(capsule):
        return UUID4.from_capsule_cvec(capsule)

    @staticmethod
    cdef UUID4 from_mem_c(UUID4_t mem):
        cdef UUID4 uuid4 = UUID4.__new__(UUID4)
        uuid4._mem = uuid4_clone(&mem)
        return uuid4

    @staticmethod
    cdef list from_capsule_cvec(object capsule):
        cdef CVec* data = <CVec*> PyCapsule_GetPointer(capsule, NULL)
        cdef UUID4_t* ptr = <UUID4_t*> data.ptr
        cdef int len = data.len
        cdef list ticks = []

        for i in range(0, len):
            ticks.append(UUID4.from_mem_c(ptr[i]))

        cvec_free(data[0])

        return ticks
