#!/bin/bash

# Loop to run the commands 10 times
for i in $(seq 1 10); do
  # Run the command and redirect output to text files
  RUSTFLAGS="-Ctarget-cpu=native" cargo run --example bench_prover --release --features parallel > "test-data/data-invRate2-${i}.txt"
done

