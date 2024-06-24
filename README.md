Experiments for implementing a synchronous deterministic message bus in Rust that interops with Python via Pyo3.

Currently it supports only one communication pattern that is p2p send. On top of it there are two events -
* Single Send Event - Send an event from A to B
* Chain Send Event - Send an event starting from A through all handlers and stopping before A.

There are four examples -
* basic - simple p2p send
* chain - simple chain send
* shared reference - handlers share a reference to the same underlying data and modify it
* pyo3_example.py - lib.rs is exported as a python library and used in the python file to interop between rust and python handlers

Before running the pyo3_example.py, build the project using `maturin develop` command to export the rust logic as a python package.
