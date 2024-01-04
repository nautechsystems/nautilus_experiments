from experiments.data.rust.core cimport logger_info
from experiments.data.rust.core cimport logger_init
from experiments.data.string cimport pystr_to_cstr

cdef class InternalLogger:
    @staticmethod
    cdef init():
        logger_init()

    @staticmethod
    cdef info(str message):
        logger_info(pystr_to_cstr(message))

cdef class CyLogger:
    @staticmethod
    def info(message):
        InternalLogger.info(message)

    @staticmethod
    def init():
        InternalLogger.init()
