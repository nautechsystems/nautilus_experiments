
from cpython.unicode cimport PyUnicode_AsUTF8String
from cpython.unicode cimport PyUnicode_FromString

from experiments.data.rust.core cimport cstr_free


cdef inline str cstr_to_pystr(const char* ptr):
    cdef str obj = PyUnicode_FromString(ptr)
    cstr_free(ptr)
    return obj


cdef inline const char* pystr_to_cstr(str value):
    cdef bytes utf8_bytes = PyUnicode_AsUTF8String(value)
    cdef char* cstr = utf8_bytes
    return cstr
