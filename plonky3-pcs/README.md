# Benchmarking Polynomial Commitment Schemes
This repository benchmarks Polynomial Commitment Schemes (PCS). The benchmarking suite evaluates:
- Commit Phase
- Open Phase
- Verification Phase

The benchmarks are executed with different `log_blowup` and `num_queries` configurations:
- `{log_blowup: 1, num_queries: 256}`
- `{log_blowup: 3, num_queries: 64}`

The trace used in the benchmark has a dimension of 19 bits for rows and 11 bits for columns, approximating 4GB of data.

## Running the Benchmark
To run the benchmarks, use:
```bash
RUSTFLAGS="-Ctarget-cpu=native" cargo bench --features parallel
```

## Running the Tests
Additionally, PCS tests are implemented:
```bash
RUSTFLAGS="-Ctarget-cpu=native" cargo test --release  --features parallel -- --nocapture
```
