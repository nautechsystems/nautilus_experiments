from experiments.data.rust.core cimport logger_info
from experiments.data.rust.core cimport logger_debug
from experiments.data.string cimport pystr_to_cstr

cdef class InternalLogger:
    @staticmethod
    cdef info(str message):
        logger_info(pystr_to_cstr(message))

    @staticmethod
    cdef debug(str message):
        logger_debug(pystr_to_cstr(message))

cdef class CyLogger:
    def info(message):
        InternalLogger.info(message)

    def debug(message):
        InternalLogger.debug(message)
