cdef class InternalLogger:
    @staticmethod
    cdef info(str message)

    @staticmethod
    cdef error(str message)

    @staticmethod
    cdef debug(str message)

cdef class CyLogger:
    pass
