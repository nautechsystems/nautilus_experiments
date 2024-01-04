from experiments.data.rust.core cimport logger_info
from experiments.data.rust.core cimport logger_debug
from experiments.data.rust.core cimport logger_error
from experiments.data.rust.core cimport pystr_to_cstr

cdef class InternalLogger:
    @staticmethod
    cdef info(str message):
        logger_info(pystr_to_cstr(message))

    @staticmethod
    cdef error(str message):
        logger_error(pystr_to_cstr(message))

    @staticmethod
    cdef debug(str message):
        logger_debug(pystr_to_cstr(message))

cdef class CyLogger:
    def info(message):
        InternalLogger.info(message)

    def error(message):
        InternalLogger.error(message)

    def debug(message):
        InternalLogger.debug(message)
