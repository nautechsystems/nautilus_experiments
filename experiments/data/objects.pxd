from experiments.data.rust.core cimport UUID4_t
from cpython.object cimport PyObject
from cpython.ref cimport Py_XDECREF

cdef class UUID4:
    cdef UUID4_t _mem

    cdef str to_str(self)

    @staticmethod
    cdef UUID4 from_mem_c(UUID4_t raw)

cdef inline str pyobj_to_str(PyObject* ptr):
    cdef PyObject* str_obj = ptr
    cdef str str_value = <str>str_obj
    Py_XDECREF(str_obj)
    return str_value
