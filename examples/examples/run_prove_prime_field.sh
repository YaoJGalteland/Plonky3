#!/bin/bash

# Loop to run the commands 10 times
for i in $(seq 1 10); do
  # Run the command and redirect output to text files
  RUSTFLAGS="-Ctarget-cpu=native" cargo run --example prove_prime_field_31 --release --features parallel -- --field koala-bear --objective poseidon-2-permutations --log-trace-length 19 --discrete-fourier-transform radix-2-dit-parallel --merkle-hash poseidon-2 > "test-data/data-poseidon2-${i}.txt"
  RUSTFLAGS="-Ctarget-cpu=native" cargo run --example prove_prime_field_31 --release --features parallel -- --field koala-bear --objective poseidon-2-permutations --log-trace-length 19 --discrete-fourier-transform radix-2-dit-parallel --merkle-hash keccak-f > "test-data/data-keccak-f-${i}.txt"
done

