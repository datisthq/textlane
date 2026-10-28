#!/usr/bin/env bash
# Builds a wheel with the crate's `testing` feature and runs the conformance suite against it.
set -euo pipefail
cd "$(dirname "$0")/.."
bash scripts/copy-skills.sh
rm -rf build/test
uvx maturin@1.15.0 build --features python,testing --out build/test
wheel=$(ls build/test/*.whl)
uv run --no-project --with "$wheel" --with pytest==8.4.2 pytest -q ./-test/conformance_unit.py
