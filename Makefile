install:
	poetry install

build:
	poetry run python build.py

clean:
	git clean -fxd

cargo-update:
	(cd nautilus_core && cargo update)

cargo-test:
	(cd nautilus_core && cargo test)

update:
	(cd nautilus_core && cargo update)
	poetry update
