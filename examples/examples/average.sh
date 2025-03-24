#!/bin/bash

# Initialize sum and count variables for each metric for both Poseidon2 and Keccak
total_prove_time_poseidon2=0
total_commit_time_poseidon2=0
total_proof_size_poseidon2=0
total_prove_time_keccak=0
total_commit_time_keccak=0
total_proof_size_keccak=0

count_prove_poseidon2=0
count_commit_poseidon2=0
count_proof_size_poseidon2=0
count_prove_keccak=0
count_commit_keccak=0
count_proof_size_keccak=0

# Function to compute averages
compute_average() {
  if [ "$2" -gt 0 ]; then
    echo "scale=2; $1 / $2" | bc
  else
    echo "N/A"
  fi
}

# Loop through all Poseidon2 log files (e.g., 1 to 10)
for i in $(seq 1 10); do
  file_poseidon2="test-data/data-poseidon2-${i}.txt"

  # Extract times using grep and sed for Poseidon2
  prove_time_poseidon2=$(grep 'prove \[' "$file_poseidon2" | sed -E 's/.*prove \[ ([0-9.]+)s.*/\1/')
  commit_time_poseidon2=$(grep 'commit to trace data' "$file_poseidon2" | sed -E 's/.*commit to trace data \[ ([0-9.]+)s.*/\1/')
  proof_size_poseidon2=$(grep 'Proof size:' "$file_poseidon2" | sed -E 's/.*Proof size: ([0-9]+) bytes.*/\1/')

  # Sum up values if they exist for Poseidon2
  if [[ -n "$prove_time_poseidon2" ]]; then
    total_prove_time_poseidon2=$(echo "$total_prove_time_poseidon2 + $prove_time_poseidon2" | bc)
    count_prove_poseidon2=$((count_prove_poseidon2 + 1))
  fi

  if [[ -n "$commit_time_poseidon2" ]]; then
    total_commit_time_poseidon2=$(echo "$total_commit_time_poseidon2 + $commit_time_poseidon2" | bc)
    count_commit_poseidon2=$((count_commit_poseidon2 + 1))
  fi

  if [[ -n "$proof_size_poseidon2" ]]; then
    total_proof_size_poseidon2=$(echo "$total_proof_size_poseidon2 + $proof_size_poseidon2" | bc)
    count_proof_size_poseidon2=$((count_proof_size_poseidon2 + 1))
  fi
done

# Loop through all Keccak log files (e.g., 1 to 10)
for i in $(seq 1 10); do
  file_keccak="test-data/data-keccak-f-${i}.txt"

  # Extract times using grep and sed for Keccak
  prove_time_keccak=$(grep 'prove \[' "$file_keccak" | sed -E 's/.*prove \[ ([0-9.]+)s.*/\1/')
  commit_time_keccak=$(grep 'commit to trace data' "$file_keccak" | sed -E 's/.*commit to trace data \[ ([0-9.]+)s.*/\1/')
  proof_size_keccak=$(grep 'Proof size:' "$file_keccak" | sed -E 's/.*Proof size: ([0-9]+) bytes.*/\1/')

  # Sum up values if they exist for Keccak
  if [[ -n "$prove_time_keccak" ]]; then
    total_prove_time_keccak=$(echo "$total_prove_time_keccak + $prove_time_keccak" | bc)
    count_prove_keccak=$((count_prove_keccak + 1))
  fi

  if [[ -n "$commit_time_keccak" ]]; then
    total_commit_time_keccak=$(echo "$total_commit_time_keccak + $commit_time_keccak" | bc)
    count_commit_keccak=$((count_commit_keccak + 1))
  fi

  if [[ -n "$proof_size_keccak" ]]; then
    total_proof_size_keccak=$(echo "$total_proof_size_keccak + $proof_size_keccak" | bc)
    count_proof_size_keccak=$((count_proof_size_keccak + 1))
  fi
done

# Calculate averages for Poseidon2
average_prove_time_poseidon2=$(compute_average $total_prove_time_poseidon2 $count_prove_poseidon2)
average_commit_time_poseidon2=$(compute_average $total_commit_time_poseidon2 $count_commit_poseidon2)
average_proof_size_poseidon2=$(compute_average $total_proof_size_poseidon2 $count_proof_size_poseidon2)

# Calculate averages for Keccak
average_prove_time_keccak=$(compute_average $total_prove_time_keccak $count_prove_keccak)
average_commit_time_keccak=$(compute_average $total_commit_time_keccak $count_commit_keccak)
average_proof_size_keccak=$(compute_average $total_proof_size_keccak $count_proof_size_keccak)

# Print results for Poseidon2
echo "Poseidon2 - Average Prove time: $average_prove_time_poseidon2 s"
echo "Poseidon2 - Average Commit to Trace Data time: $average_commit_time_poseidon2 s"
echo "Poseidon2 - Average Proof size: $average_proof_size_poseidon2 bytes"

# Print results for Keccak
echo "Keccak - Average Prove time: $average_prove_time_keccak s"
echo "Keccak - Average Commit to Trace Data time: $average_commit_time_keccak s"
echo "Keccak - Average Proof size: $average_proof_size_keccak bytes"
