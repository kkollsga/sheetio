.PHONY: dev test lint fmt cov clean

dev:
	maturin develop --release

test:
	pytest tests/ -x

lint:
	cargo fmt -- --check
	cargo clippy -- -D warnings
	ruff format --check .
	ruff check .

fmt:
	cargo fmt
	ruff format .
	ruff check --fix .

cov:
	pytest tests/ --cov=sheetio --cov-report=term-missing

clean:
	cargo clean
	rm -rf wheels/ dist/ *.egg-info build/
