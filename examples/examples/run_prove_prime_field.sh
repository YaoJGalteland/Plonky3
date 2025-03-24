#!/bin/bash

# Loop to run the command 10 times
for i in $(seq 1 10); do
  # Run the command and redirect output to a text file (data1.txt, data2.txt, ...)
  RUSTFLAGS="-Ctarget-cpu=native" cargo run --example prove_prime_field_31 --release --features parallel -- --field koala-bear --objective keccak-f-permutations --log-trace-length 19 --discrete-fourier-transform radix-2-dit-parallel --merkle-hash poseidon-2 > "test-data/data-koala-bear-keccak-${i}.txt"
done

