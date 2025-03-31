#!/bin/bash

# Get total CPU cores
TOTAL_CORES=$(nproc)

# Start tracking CPU usage in the background
mpstat -P ALL 1 > cpu_usage.log &

# Get background process PID
MPSTAT_PID=$!

# Run Rust benchmark
RUSTFLAGS="-Ctarget-cpu=native" cargo bench --features parallel

# Stop mpstat
kill $MPSTAT_PID

# Process the log file
AVG_IDLE=$(awk '/^Average:/ && $2 == "all" {print $NF}' cpu_usage.log)

# Calculate Average Cores Used
AVG_CORES_USED=$(echo "$TOTAL_CORES * (1 - $AVG_IDLE / 100)" | bc -l)

echo "Average CPU cores used: $AVG_CORES_USED"
