#!/bin/bash
success_count=0
failed_count=0

# Initialize string variables to store the names of success and fail folders
success_folders=""
failed_folders=""

for d in */; do
    echo "${d}"
    (cd "${d}" && cargo +nightly-2023-06-02 check >/dev/null 2>&1)

    if [ $? -eq 0 ]; then
        (cd "${d}" && cargo +nightly-2023-06-02 clippy -- -A clippy::all -A warnings -D non_snake_case -D non_upper_case_globals -D non_camel_case_types -D missing_debug_implementations -D clippy::wrong_self_convention -D clippy::missing_errors_doc -D clippy::missing_panics_doc -D clippy::undocumented_unsafe_blocks -D clippy::missing_safety_doc -D clippy::approx_constant -D clippy::borrow_interior_mutable_const -D clippy::declare_interior_mutable_const -D clippy::default_numeric_fallback -D clippy::bool_assert_comparison -D clippy::bool_comparison -D clippy::blocks_in_if_conditions -D clippy::arithmetic_side_effects -D clippy::needless_range_loop -D clippy::recursive_format_impl -D clippy::precedence -D clippy::bad_bit_mask -D clippy::match_overlapping_arm -D clippy::large_types_passed_by_value -D clippy::derived_hash_with_manual_eq -D clippy::derive_ord_xor_partial_ord -D clippy::from_over_into -D clippy::unwrap_used -D clippy::expect_used -D clippy::wildcard_imports -D clippy::mut_from_ref -D clippy::mutex_atomic -D clippy::mutex_integer -D clippy::redundant_allocation)
        success_count=$((success_count + 1))
        success_folders="${success_folders}${d%/}\n"
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
