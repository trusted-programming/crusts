#!/bin/sh

warnings_string=$(grep -i 'error:' log.txt | grep -iv 'could not compile')

warnings=$(echo "$warnings_string" | wc -l)

echo "Warnings: $warnings"

# read line count from the directory
lines=$(tokei -t=Rust --output=json ${d} | jq -r '.Rust.code')

echo "Lines: $lines"

# getting number of lines in thousands
lines_in_k=$(echo "scale=3; $lines / 1000" | bc -l)

# calculate number of warnings per 1000 lines
warnings_per_k=$(echo "scale=3; $warnings / $lines_in_k" | bc -l)

echo "KLOC: $warnings_per_k"

warnings_types=$(echo "$warnings_string" | sort | uniq -c | sort -nr)

echo "Warning types:"
echo "$warnings_types"
