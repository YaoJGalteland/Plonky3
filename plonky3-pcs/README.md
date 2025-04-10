# Benchmarking Polynomial Commitment Schemes
This repository benchmarks Fri PCS. The benchmarking suite evaluates:
- Commit Phase
- Open Phase

The trace used in the benchmark has a dimension of 19 bits for rows and 11 bits for columns, approximating 4GB of data.

## Running the Benchmark
Run:
```bash
cd Plonky3/plonky3-pcs 
rustup default nightly
RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C target-feature=+avx512f" cargo +nightly bench --features "nightly-features" --features parallel
```

## Running the Tests
Additionally, PCS tests are implemented.

Test Fri PCS:
```bash
RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C target-feature=+avx512f" cargo +nightly test --release --features "nightly-features"  --features parallel
```
