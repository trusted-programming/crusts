#!/bin/sh

# Initialize count variables for success and fail counts
success_count=0
failed_count=0

# Initialize string variables to store the names of success and fail folders
success_folders=""
failed_folders=""

# Loop over each directory (denoted by */) in the current directory
for d in */; do
    # Print the current working directory
    echo "Current directory: $(d)"

    # Change to directory "${d}" and run cargo check.
    # Parentheses are used to run these commands in a subshell,
    # which means the current directory is only changed for these commands.
    (cd "${d}" && cargo +nightly-2023-06-02 check)

    # Check the exit status of the last command with "$?".
    # If it's 0 (which means the command was successful), increment the success count
    # and add the directory name to the success folders string.
    if [ $? -eq 0 ]; then
        success_count=$((success_count + 1))
        success_folders="${success_folders}${d%/}\n"
    # If the command was not successful, increment the fail count
    # and add the directory name to the fail folders string.
    else
        failed_count=$((failed_count + 1))
        failed_folders="${failed_folders}${d%/}\n"
    fi
done

# Calculate the percentage of successful runs
# First, calculate the total number of runs as the sum of successful and failed runs
total_runs=$((success_count + failed_count))

# If there were any runs, calculate the percentage of successful runs as (successful runs / total runs) * 100.
# Use the bc command to do the division and multiplication, and set the scale to 2 to get two decimal places.
# If there were no runs, set the percentage to 0.00 to prevent division by zero.
if [ $total_runs -gt 0 ]; then
    success_percentage=$(echo "scale=2; ($success_count / $total_runs) * 100" | bc)
else
    success_percentage=0.00
fi

# Print the names of the success and fail folders, and the number of successful and failed runs
echo -e "Success folders:\n$success_folders"
echo -e "Failed folders:\n$failed_folders"

echo "Total runs: $total_runs"
echo "Successful runs: $success_count"
echo "Failed runs: $failed_count"

# Print the percentage of successful runs
echo "Successful runs percentage: $success_percentage%"
