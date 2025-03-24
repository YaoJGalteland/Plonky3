#!/bin/bash

# Initialize total time and count for each activity for Poseidon2 and Keccak-F
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

total_prove_time_keccak_f=0
total_commit_time_keccak_f=0
total_open_time_keccak_f=0
total_verify_time_keccak_f=0
total_proof_size_keccak_f=0

count_prove_keccak_f=0
count_commit_keccak_f=0
count_open_keccak_f=0
count_verify_keccak_f=0
count_proof_size_keccak_f=0

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
  # Process Poseidon2 log file
  file_poseidon2="test-data/data-poseidon2-${i}.txt"

  # Extract times using grep and sed for specific activities in Poseidon2
  prove_time=$(grep 'prove \[' "$file_poseidon2" | sed -E 's/.*prove \[ ([0-9.]+ms?|[0-9.]+s).*$/\1/')
  commit_time=$(grep 'commit to trace data' "$file_poseidon2" | sed -E 's/.*commit to trace data \[ ([0-9.]+ms?|[0-9.]+s).*$/\1/')
  open_time=$(grep 'open \[' "$file_poseidon2" | sed -E 's/.*open \[ ([0-9.]+ms?|[0-9.]+s).*$/\1/')
  verify_time=$(grep 'verify \[' "$file_poseidon2" | sed -E 's/.*verify \[ ([0-9.]+ms?|[0-9.]+s).*$/\1/')
  proof_size=$(grep 'Proof size:' "$file_poseidon2" | sed -E 's/.*Proof size: ([0-9]+) bytes.*/\1/')

  # Process prove time
  if [[ -n "$prove_time" ]]; then
    prove_time_in_sec=$(convert_to_seconds "$prove_time")
    total_prove_time_poseidon2=$(awk "BEGIN {print $total_prove_time_poseidon2 + $prove_time_in_sec}")
    count_prove_poseidon2=$((count_prove_poseidon2 + 1))
  fi

  # Process commit time
  if [[ -n "$commit_time" ]]; then
    commit_time_in_sec=$(convert_to_seconds "$commit_time")
    total_commit_time_poseidon2=$(awk "BEGIN {print $total_commit_time_poseidon2 + $commit_time_in_sec}")
    count_commit_poseidon2=$((count_commit_poseidon2 + 1))
  fi

  # Process open time
  if [[ -n "$open_time" ]]; then
    open_time_in_sec=$(convert_to_seconds "$open_time")
    total_open_time_poseidon2=$(awk "BEGIN {print $total_open_time_poseidon2 + $open_time_in_sec}")
    count_open_poseidon2=$((count_open_poseidon2 + 1))
  fi

  # Process verify time
  if [[ -n "$verify_time" ]]; then
    verify_time_in_sec=$(convert_to_seconds "$verify_time")
    total_verify_time_poseidon2=$(awk "BEGIN {print $total_verify_time_poseidon2 + $verify_time_in_sec}")
    count_verify_poseidon2=$((count_verify_poseidon2 + 1))
  fi

  # Process proof size
  if [[ -n "$proof_size" ]]; then
    total_proof_size_poseidon2=$(echo "$total_proof_size_poseidon2 + $proof_size" | bc)
    count_proof_size_poseidon2=$((count_proof_size_poseidon2 + 1))
  fi
done

# Loop through all Keccak-F log files (e.g., 1 to 10)
for i in $(seq 1 10); do
  # Process Keccak-F log file
  file_keccak_f="test-data/data-keccak-f-${i}.txt"

  # Extract times using grep and sed for specific activities in Keccak-F
  prove_time=$(grep 'prove \[' "$file_keccak_f" | sed -E 's/.*prove \[ ([0-9.]+ms?|[0-9.]+s).*$/\1/')
  commit_time=$(grep 'commit to trace data' "$file_keccak_f" | sed -E 's/.*commit to trace data \[ ([0-9.]+ms?|[0-9.]+s).*$/\1/')
  open_time=$(grep 'open \[' "$file_keccak_f" | sed -E 's/.*open \[ ([0-9.]+ms?|[0-9.]+s).*$/\1/')
  verify_time=$(grep 'verify \[' "$file_keccak_f" | sed -E 's/.*verify \[ ([0-9.]+ms?|[0-9.]+s).*$/\1/')
  proof_size=$(grep 'Proof size:' "$file_keccak_f" | sed -E 's/.*Proof size: ([0-9]+) bytes.*/\1/')

  # Process prove time
  if [[ -n "$prove_time" ]]; then
    prove_time_in_sec=$(convert_to_seconds "$prove_time")
    total_prove_time_keccak_f=$(awk "BEGIN {print $total_prove_time_keccak_f + $prove_time_in_sec}")
    count_prove_keccak_f=$((count_prove_keccak_f + 1))
  fi

  # Process commit time
  if [[ -n "$commit_time" ]]; then
    commit_time_in_sec=$(convert_to_seconds "$commit_time")
    total_commit_time_keccak_f=$(awk "BEGIN {print $total_commit_time_keccak_f + $commit_time_in_sec}")
    count_commit_keccak_f=$((count_commit_keccak_f + 1))
  fi

  # Process open time
  if [[ -n "$open_time" ]]; then
    open_time_in_sec=$(convert_to_seconds "$open_time")
    total_open_time_keccak_f=$(awk "BEGIN {print $total_open_time_keccak_f + $open_time_in_sec}")
    count_open_keccak_f=$((count_open_keccak_f + 1))
  fi

  # Process verify time
  if [[ -n "$verify_time" ]]; then
    verify_time_in_sec=$(convert_to_seconds "$verify_time")
    total_verify_time_keccak_f=$(awk "BEGIN {print $total_verify_time_keccak_f + $verify_time_in_sec}")
    count_verify_keccak_f=$((count_verify_keccak_f + 1))
  fi

  # Process proof size
  if [[ -n "$proof_size" ]]; then
    total_proof_size_keccak_f=$(echo "$total_proof_size_keccak_f + $proof_size" | bc)
    count_proof_size_keccak_f=$((count_proof_size_keccak_f + 1))
  fi
done

# Compute averages for prove, commit, open, verify, and proof size
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
echo "Poseidon2 Results:"
echo "Average Prove time: $(compute_average $total_prove_time_poseidon2 $count_prove_poseidon2) seconds"
echo "Average Commit to Trace Data time: $(compute_average $total_commit_time_poseidon2 $count_commit_poseidon2) seconds"
echo "Average Open time: $(compute_average $total_open_time_poseidon2 $count_open_poseidon2) seconds"
echo "Average Verify time: $(compute_average $total_verify_time_poseidon2 $count_verify_poseidon2) seconds"
echo "Average Proof size: $(compute_average $total_proof_size_poseidon2 $count_proof_size_poseidon2) bytes"

# Print results for Keccak-F
echo "Keccak-F Results:"
echo "Average Prove time: $(compute_average $total_prove_time_keccak_f $count_prove_keccak_f) seconds"
echo "Average Commit to Trace Data time: $(compute_average $total_commit_time_keccak_f $count_commit_keccak_f) seconds"
echo "Average Open time: $(compute_average $total_open_time_keccak_f $count_open_keccak_f) seconds"
echo "Average Verify time: $(compute_average $total_verify_time_keccak_f $count_verify_keccak_f) seconds"
echo "Average Proof size: $(compute_average $total_proof_size_keccak_f $count_proof_size_keccak_f) bytes"
