Soft-float library that intends to be a straightforward reference implementation of IEEE 754.

## Installation for use from Rust

Add to your `Cargo.toml`:

```toml
[dependencies.simple-soft-float]
version = "0.1"
```

## Installation for use from Python

Install Rust using [rustup.rs](https://rustup.rs).

Create CPython 3.6 to 3.7 virtualenv (not sure if 3.8 is supported yet).

Install Python bindings build tool:
```bash
pip install maturin
```

Get source:
```bash
git clone https://salsa.debian.org/Kazan-team/simple-soft-float.git
cd simple-soft-float
```

Change source dir to use specific version of Rust nightly:
(must be in `simple-soft-float` dir):
```bash
rustup override set nightly-2019-07-19
```

Build and Test (like `setup.py develop`):
```bash
cargo test --features python # runs tests from Rust
# build and install to python
maturin develop --cargo-extra-args="--features python-extension"
python -m unittest # runs smoke tests from Python
```

Build Rust docs:
```bash
cargo doc --features python # ignore warning about rand_core name collision
open docs in default browser:
xdg-open target/doc/simple_soft_float/struct.DynamicFloat.html
```

Build Python docs:
```bash
pip install pdoc3
pdoc3 simple_soft_float --html -o target/python-docs
xdg-open target/python-docs/simple_soft_float.html
```
