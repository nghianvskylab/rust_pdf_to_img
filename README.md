## Build Python Package (with maturin)
```
maturin build --release -m  Cargo.toml
```

## Import in Python

 After building, install the wheel file in your Python environment:
```
cd polar-server
uv pip install polar-preprocess/target/wheels/xxx.whl
 ```
