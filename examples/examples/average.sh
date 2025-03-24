#!/bin/bash

# Initialize total time and count for each activity
total_prove_time=0
total_commit_time=0
total_open_time=0
total_proof_size=0

count_prove=0
count_commit=0
count_open=0
count_proof_size=0

# Function to convert time to seconds (milliseconds to seconds)
convert_to_seconds() {
  local time_value="$1"

  if [[ $time_value == *ms ]]; then
    # Time in milliseconds, convert to seconds
    echo "$(awk "BEGIN {print ${time_value%ms} / 1000}")"
  else
    # Time in seconds, remove 's' suffix
    echo "${time_value%s}"
  fi
}

# Loop through all Poseidon2 log files (e.g., 1 to 10)
for i in $(seq 1 10); do
  file_poseidon2="test-data/data-poseidon2-${i}.txt"

  # Extract times using grep and sed for specific activities in Poseidon2
  prove_time=$(grep 'prove \[' "$file_poseidon2" | sed -E 's/.*prove \[ ([0-9.]+ms?|[0-9.]+s).*$/\1/')
  commit_time=$(grep 'commit to trace data' "$file_poseidon2" | sed -E 's/.*commit to trace data \[ ([0-9.]+ms?|[0-9.]+s).*$/\1/')
  open_time=$(grep 'open \[' "$file_poseidon2" | sed -E 's/.*open \[ ([0-9.]+ms?|[0-9.]+s).*$/\1/')
  proof_size=$(grep 'Proof size:' "$file_poseidon2" | sed -E 's/.*Proof size: ([0-9]+) bytes.*/\1/')

  # Process prove time
  if [[ -n "$prove_time" ]]; then
    prove_time_in_sec=$(convert_to_seconds "$prove_time")
    total_prove_time=$(awk "BEGIN {print $total_prove_time + $prove_time_in_sec}")
    count_prove=$((count_prove + 1))
  fi

  # Process commit time
  if [[ -n "$commit_time" ]]; then
    commit_time_in_sec=$(convert_to_seconds "$commit_time")
    total_commit_time=$(awk "BEGIN {print $total_commit_time + $commit_time_in_sec}")
    count_commit=$((count_commit + 1))
  fi

  # Process open time
  if [[ -n "$open_time" ]]; then
    open_time_in_sec=$(convert_to_seconds "$open_time")
    total_open_time=$(awk "BEGIN {print $total_open_time + $open_time_in_sec}")
    count_open=$((count_open + 1))
  fi

  # Process proof size
  if [[ -n "$proof_size" ]]; then
    total_proof_size=$(echo "$total_proof_size + $proof_size" | bc)
    count_proof_size=$((count_proof_size + 1))
  fi
done

# Compute averages for prove, commit, open, and proof size
compute_average() {
  local total=$1
  local count=$2
  if [ "$count" -gt 0 ]; then
    echo "$(awk "BEGIN {print $total / $count}")"
  else
    echo "N/A"
  fi
}

# Print results for Poseidon2
echo "Average Prove time: $(compute_average $total_prove_time $count_prove) seconds"
echo "Average Commit to Trace Data time: $(compute_average $total_commit_time $count_commit) seconds"
echo "Average Open time: $(compute_average $total_open_time $count_open) seconds"
echo "Average Proof size: $(compute_average $total_proof_size $count_proof_size) bytes"
