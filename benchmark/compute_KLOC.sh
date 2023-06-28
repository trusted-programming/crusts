#!/bin/sh

# read error count from the log file
errors=$(cat log.txt | grep -c 'error:')

# read line count from the directory
lines=$(tokei -t=Rust --output=json ${d} | jq -r '.Rust.code')

# getting number of lines in thousands
lines_in_k=$(echo "scale=3; $lines / 1000" | bc -l)

# calculate number of errors per 1000 lines
errors_per_k=$(echo "scale=3; $errors / $lines_in_k" | bc -l)

echo "Warnings: $errors"
echo "Lines: $lines"
echo ""
echo "KLOC: $errors_per_k"
