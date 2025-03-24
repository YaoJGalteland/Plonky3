#!/bin/bash

# Initialize sum and count variables for each metric for Poseidon2 and Keccak
total_prove_time_poseidon2=0
total_commit_time_poseidon2=0
total_open_time_poseidon2=0
total_verify_time_poseidon2=0
total_proof_size_poseidon2=0

count_prove_poseidon2=0
count_commit_poseidon2=0
count_open_poseidon2=0
count_verify_poseidon2=0
count_proof_size_poseidon2=0

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
  prove_time_poseidon2=$(grep 'prove \[' "$file_poseidon2" | sed -E 's/.*prove \[ ([0-9.]+)ms.*/\1/')
  commit_time_poseidon2=$(grep 'commit to trace data' "$file_poseidon2" | sed -E 's/.*commit to trace data \[ ([0-9.]+)ms.*/\1/')
  open_time_poseidon2=$(grep 'open' "$file_poseidon2" | sed -E 's/.*open \[ ([0-9.]+)ms.*/\1/')
  verify_time_poseidon2=$(grep 'verify \[' "$file_poseidon2" | sed -E 's/.*verify \[ ([0-9.]+)ms.*/\1/')
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

  if [[ -n "$open_time_poseidon2" ]]; then
    total_open_time_poseidon2=$(echo "$total_open_time_poseidon2 + $open_time_poseidon2" | bc)
    count_open_poseidon2=$((count_open_poseidon2 + 1))
  fi

  if [[ -n "$verify_time_poseidon2" ]]; then
    total_verify_time_poseidon2=$(echo "$total_verify_time_poseidon2 + $verify_time_poseidon2" | bc)
    count_verify_poseidon2=$((count_verify_poseidon2 + 1))
  fi

  if [[ -n "$proof_size_poseidon2" ]]; then
    total_proof_size_poseidon2=$(echo "$total_proof_size_poseidon2 + $proof_size_poseidon2" | bc)
    count_proof_size_poseidon2=$((count_proof_size_poseidon2 + 1))
  fi
done

# Calculate averages for Poseidon2
average_prove_time_poseidon2=$(compute_average $total_prove_time_poseidon2 $count_prove_poseidon2)
average_commit_time_poseidon2=$(compute_average $total_commit_time_poseidon2 $count_commit_poseidon2)
average_open_time_poseidon2=$(compute_average $total_open_time_poseidon2 $count_open_poseidon2)
average_verify_time_poseidon2=$(compute_average $total_verify_time_poseidon2 $count_verify_poseidon2)
average_proof_size_poseidon2=$(compute_average $total_proof_size_poseidon2 $count_proof_size_poseidon2)

# Print results for Poseidon2
echo "Poseidon2 - Average Prove time: $average_prove_time_poseidon2 ms"
echo "Poseidon2 - Average Commit to Trace Data time: $average_commit_time_poseidon2 ms"
echo "Poseidon2 - Average Open time: $average_open_time_poseidon2 ms"
echo "Poseidon2 - Average Verify time: $average_verify_time_poseidon2 ms"
echo "Poseidon2 - Average Proof size: $average_proof_size_poseidon2 bytes"
