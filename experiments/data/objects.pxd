from experiments.data.rust.core cimport UUID4_t

cdef class UUID4:
    cdef UUID4_t _uuid4

    cdef readonly str value
    """The UUID4 value.\n\n:returns: `str`"""

    cdef UUID4_t _uuid4_from_pystr(self, str value) except *
