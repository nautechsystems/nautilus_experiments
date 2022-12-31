
def test_pystr_cstr_conversions():
    cdef str input1 = "hello, world 1"
    cstr_to_pystr(pystr_to_cstr(input1))
    # cdef str input2 = "hello, world 2"
    # cdef str input3 = "hello, world 3"
    # cdef const char* output1 = pystr_to_cstr(input1)
    # cdef const char* output2 = pystr_to_cstr(input2)
    # cdef const char* output3 = pystr_to_cstr(input3)
    # cstr_to_pystr(output1)
    # cstr_to_pystr(output2)
    # cstr_to_pystr(output3)
