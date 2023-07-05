#!/bin/sh
success_count=0
failed_count=0

# Initialize string variables to store the names of success and fail folders
success_folders=""
failed_folders=""

for d in */; do
    # Print the current working directory
    echo "Started directory: ${d}"

    # Run crusts
    (cd "${d}" && crusts) >>crusts.log 2>&1

    # REMOVE UNSAFE WARNINGS
    # Use find to find all .rs files recursively in the given folder
    find "$d" -name "*.rs" -print0 | while IFS= read -r -d '' file; do
        # Use grep to check if the file contains the keyword "unsafe", if not skip the file
        if grep -q "unsafe" "$file"; then
            # Use sed to add comment above each line containing "unsafe"
            sed -i '/unsafe/ i\// SAFETY: machine generated unsafe code' "$file"
        fi

        if grep -q "struct" "$file"; then
            # Use sed to add comment above each line containing "unsafe"
            sed -i '/struct/ i\#[derive(Debug)]' "$file"
        fi
    done

    # run clippy fix
    (cd "${d}" && cargo +nightly-2023-06-02 clippy --fix --allow-dirty --allow-no-vcs) >>clippy_fix.log 2>&1

    # filter out non compiling
    if [ $? -eq 0 ]; then
        success_count=$((success_count + 1))
        success_folders="${success_folders}${d%/}\n"

        # run clippy with 30 warninings denied
        (cd "${d}" && cargo +nightly-2023-06-02 clippy -- -A clippy::all -A warnings -D non_snake_case -D non_upper_case_globals -D non_camel_case_types -D missing_debug_implementations -D clippy::wrong_self_convention -D clippy::missing_errors_doc -D clippy::missing_panics_doc -D clippy::undocumented_unsafe_blocks -D clippy::missing_safety_doc -D clippy::approx_constant -D clippy::borrow_interior_mutable_const -D clippy::declare_interior_mutable_const -D clippy::default_numeric_fallback -D clippy::bool_assert_comparison -D clippy::bool_comparison -D clippy::blocks_in_if_conditions -D clippy::arithmetic_side_effects -D clippy::needless_range_loop -D clippy::recursive_format_impl -D clippy::precedence -D clippy::bad_bit_mask -D clippy::match_overlapping_arm -D clippy::large_types_passed_by_value -D clippy::derived_hash_with_manual_eq -D clippy::derive_ord_xor_partial_ord -D clippy::from_over_into -D clippy::unwrap_used -D clippy::expect_used -D clippy::wildcard_imports -D clippy::mut_from_ref -D clippy::mutex_atomic -D clippy::mutex_integer -D clippy::redundant_allocation) >>clippy.log 2>&1
    else
        failed_count=$((failed_count + 1))
        failed_folders="${failed_folders}${d%/}\n"
    fi
done

total_runs=$((success_count + failed_count))

if [ $total_runs -gt 0 ]; then
    success_percentage=$(echo "scale=2; ($success_count / $total_runs) * 100" | bc)
else
    success_percentage=0.00
fi

warnings_string=$(grep -i 'error:' clippy.log | grep -iv 'could not compile')

warnings=$(echo "$warnings_string" | wc -l)

echo "**Warnings**: $warnings" >>README.md

# read line count from the directory
lines=$(tokei -t=Rust --output=json ${d} | jq -r '.Rust.code')

echo "**Lines**: $lines" >>README.md

# getting number of lines in thousands
lines_in_k=$(echo "scale=3; $lines / 1000" | bc -l)

# calculate number of warnings per 1000 lines
warnings_per_k=$(echo "scale=3; $warnings / $lines_in_k" | bc -l)

echo "**KLOC**: $warnings_per_k" >>README.md

warnings_types=$(echo "$warnings_string" | sort | uniq -c | sort -nr)

echo "**Warning types**:" >>README.md
echo "$warnings_types" >>README.md

echo -e "**Success folders**:\n$success_folders" >>README.md
echo -e "**Failed folders**:\n$failed_folders" >>README.md

echo "**Total runs**: $total_runs" >>README.md
echo "**Successful runs**: $success_count" >>README.md
echo "**Failed runs**: $failed_count" >>README.md

echo "**Successful runs percentage**: $success_percentage%" >>README.md
