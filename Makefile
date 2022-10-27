install:
	poetry install

build:
	poetry run python build.py

clean:
	git clean -fxd

cargo-update:
	(cd experiments/core && cargo update)

cargo-test:
	(cd experiments/core && cargo test)

update:
	(cd experiments/core && cargo update)
	poetry update
