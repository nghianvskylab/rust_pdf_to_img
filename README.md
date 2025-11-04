## Build Python Package (with maturin)
```
maturin build --release -m  Cargo.toml
```

## Import in Python

 After building, install the wheel file in your Python environment:
```
cd rust_pdf_to_img
uv pip install preprocess/target/wheels/xxx.whl
 ```
