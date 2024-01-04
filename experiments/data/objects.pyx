from core import TempLogger

cdef class CyLogger:
    @staticmethod
    def info(message):
        logger = TempLogger("cython")
        logger.info(message)
