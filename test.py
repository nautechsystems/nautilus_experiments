from inner import MyError, raise_myerror

try:
    raise_myerror()
except MyError as e:
    print("Received MyError")
    print(e)
