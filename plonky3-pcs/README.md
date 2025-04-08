# Benchmarking Polynomial Commitment Schemes
This repository benchmarks Polynomial Commitment Schemes (PCS). The benchmarking suite evaluates:
- Commit Phase
- Open Phase
- Verification Phase

The benchmarks are executed with different `log_blowup` and `num_queries` configurations:
- `{log_blowup: 1, num_queries: 256}`
- `{log_blowup: 3, num_queries: 64}`

The trace used in the benchmark has a dimension of 19 bits for rows and 11 bits for columns, approximating 4GB of data.

## Benchmarking FFT with Different Feature Flags
Run:
```bash
cd Plonky3/circle 

# Single-thread, no AVX512
cargo bench --features benches_diff_flags  

# Enable AVX-512 only
RUSTFLAGS="-Ctarget-feature=+avx512f" cargo bench --features benches_diff_flags  

# AVX-512 with parallel feature
RUSTFLAGS="-Ctarget-feature=+avx512f" cargo bench --features parallel --features benches_diff_flags  

```

## Benchmarking FFT for a Practical Large Trace 
Run:
```bash
cd Plonky3/circle
RUSTFLAGS="-Ctarget-feature=+avx512f" cargo +nightly bench --features "nightly-features"  --features parallel --features benches_large_trace
```

## Benchmarking PCS
Run:
```bash
cd Plonky3/plonky3-pcs 
RUSTFLAGS="-Ctarget-feature=+avx512f" cargo +nightly bench --features "nightly-features"  --features parallel
```

## Running the Tests
Additionally, PCS tests are implemented:
```bash
RUSTFLAGS="-Ctarget-feature=+avx512f" cargo +nightly test --release --features "nightly-features"  --features parallel
```
