from experiments.data.rust.core cimport UUID4_t

cdef class UUID4:
    cdef UUID4_t _mem

    cdef readonly str value
    """The UUID4 value.\n\n:returns: `str`"""

    cdef str to_str(self)

