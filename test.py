import sys
sys.path.append("/home/twitu/Code/nautilus_experiments/.venv/lib/python3.11/site-packages")

from pyo3_test import print_hello_world

class Test:
    def __init__(self) -> None:
        pass
    
    def do(self):
        print_hello_world()
