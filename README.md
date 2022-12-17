# Nautilus Experiments

Install Python, Rust and [poetry](https://python-poetry.org/docs/) to setup the experiments repo.

```
make install
make build
cd experiments/core/ && maturin develop --features extension-module
cd ../.. && python tests/test_objects.py
```

