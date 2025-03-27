#!/bin/bash

# Initialize total time and count for each activity for invRate2
total_commit_time_invRate2=0
total_open_time_invRate2=0

count_commit_invRate2=0
count_open_invRate2=0

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

# Loop through all invRate2 log files (e.g., 1 to 10)
for i in $(seq 1 10); do
  # Process invRate2 log file
  file_invRate2="test-data/data-invRate2-${i}.txt"

  # Extract times using grep and sed for specific activities in invRate2
  commit_time=$(grep 'commit to trace data' "$file_invRate2" | sed -E 's/.*commit to trace data \[ ([0-9.]+ms?|[0-9.]+s).*$/\1/')
  open_time=$(grep 'open \[' "$file_invRate2" | sed -E 's/.*open \[ ([0-9.]+ms?|[0-9.]+s).*$/\1/')

  # Process commit time
  if [[ -n "$commit_time" ]]; then
    commit_time_in_sec=$(convert_to_seconds "$commit_time")
    total_commit_time_invRate2=$(awk "BEGIN {print $total_commit_time_invRate2 + $commit_time_in_sec}")
    count_commit_invRate2=$((count_commit_invRate2 + 1))
  fi

  # Process open time
  if [[ -n "$open_time" ]]; then
    open_time_in_sec=$(convert_to_seconds "$open_time")
    total_open_time_invRate2=$(awk "BEGIN {print $total_open_time_invRate2 + $open_time_in_sec}")
    count_open_invRate2=$((count_open_invRate2 + 1))
  fi
done

# Compute averages for commit and open time
compute_average() {
  local total=$1
  local count=$2
  if [ "$count" -gt 0 ]; then
    echo "$(awk "BEGIN {print $total / $count}")"
  else
    echo "N/A"
  fi
}

# Print results for invRate2
echo "invRate2 Results:"
echo "Average Commit to Trace Data time: $(compute_average $total_commit_time_invRate2 $count_commit_invRate2) seconds"
echo "Average Open time: $(compute_average $total_open_time_invRate2 $count_open_invRate2) seconds"
