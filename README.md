Build the wrap_ustr library as a shared object

```bash
cd wrap_ustr
cargo build --lib
cd ..
```

Build the pyo3_test crate as a pyo3 module and then as an executable binary. The binary is configured to compile while ignoring unlinked symbols and exporting all symbols for dynamic linking.

```bash
maturin develop
RUSTFLAGS="-C link-args=-Wl,--unresolved-symbols=ignore-all,--export-dynamic" cargo build
RUST_BACKTRACE=1 LD_PRELOAD="/home/twitu/Code/nautilus_experiments/wrap_ustr/target/debug/libwrap_ustr.so" target/debug/pyo3_test
```

Ths print statements show that the following works
* Linking to wrap_ustr so works because "debug function" is printed
* Ustr pointers are passed from binary to so successfully
* Message bus pointer is passed from binary to so through python successfully

FAILURE: The logic fails when trying to dereference the entries of the index map in the `get_value` call due memory misalignment issues.

```
Starting program
debug_function
Ptr: 0x59e8512b5588, len: 1
Ptr: 0x59e8512b5588, len: 1
None
Ptr: 0x59e8512b5589, len: 1
Ptr: 0x59e8512b5589, len: 1
None
Value pointer address: 0x59e88a975910
Value pointer address: 0x59e88a975910
Ptr: 0x792f9f14e6e8, len: 1
Ptr: 0x792f9f14e6e8, len: 1

thread '<unnamed>' panicked at library/core/src/panicking.rs:218:5:
unsafe precondition(s) violated: slice::from_raw_parts requires the pointer to be aligned and non-null, and the total size of the slice not to exceed `isize::MAX`
```
