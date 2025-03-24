#!/bin/bash

# Initialize sum and count variables for each metric
total_prove_time=0
total_commit_time=0
total_proof_size=0

count_prove=0
count_commit=0
count_proof_size=0

# Loop through all log files (e.g., 1 to 4)
for i in $(seq 1 10); do
  file="test-data/data-koala-bear-keccak-${i}.txt"

  # Extract times using grep and sed
  prove_time=$(grep 'prove \[' "$file" | sed -E 's/.*prove \[ ([0-9.]+)s.*/\1/')
  commit_time=$(grep 'commit to trace data' "$file" | sed -E 's/.*commit to trace data \[ ([0-9.]+)s.*/\1/')
  proof_size=$(grep 'Proof size:' "$file" | sed -E 's/.*Proof size: ([0-9]+) bytes.*/\1/')

  # Sum up values if they exist
  if [[ -n "$prove_time" ]]; then
    total_prove_time=$(echo "$total_prove_time + $prove_time" | bc)
    count_prove=$((count_prove + 1))
  fi

  if [[ -n "$commit_time" ]]; then
    total_commit_time=$(echo "$total_commit_time + $commit_time" | bc)
    count_commit=$((count_commit + 1))
  fi

  if [[ -n "$proof_size" ]]; then
    total_proof_size=$(echo "$total_proof_size + $proof_size" | bc)
    count_proof_size=$((count_proof_size + 1))
  fi
done

# Compute averages
compute_average() {
  if [ "$2" -gt 0 ]; then
    echo "scale=2; $1 / $2" | bc
  else
    echo "N/A"
  fi
}

# Calculate averages
average_prove_time=$(compute_average $total_prove_time $count_prove)
average_commit_time=$(compute_average $total_commit_time $count_commit)
average_proof_size=$(compute_average $total_proof_size $count_proof_size)

# Print results
echo "Average Prove time: $average_prove_time s"
echo "Average Commit to Trace Data time: $average_commit_time s"
echo "Average Proof size: $average_proof_size bytes"
