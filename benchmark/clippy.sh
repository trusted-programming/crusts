#!/bin/sh

n_warnings=0
code_count=0

for d in */; do
    project_n_warnings=$(cd "${d}" && cargo +nightly clippy -- -A clippy::all -A warnings -D non_snake_case -D non_upper_case_globals -D non_camel_case_types -D missing_debug_implementations -D clippy::wrong_self_convention -D clippy::missing_errors_doc -D clippy::missing_panics_doc -D clippy::undocumented_unsafe_blocks -D clippy::missing_safety_doc -D clippy::approx_constant -D clippy::borrow_interior_mutable_const -D clippy::declare_interior_mutable_const -D clippy::default_numeric_fallback -D clippy::bool_assert_comparison -D clippy::bool_comparison -D clippy::blocks_in_if_conditions -D clippy::arithmetic_side_effects -D clippy::needless_range_loop -D clippy::recursive_format_impl -D clippy::precedence -D clippy::bad_bit_mask -D clippy::match_overlapping_arm -D clippy::large_types_passed_by_value -D clippy::derived_hash_with_manual_eq -D clippy::derive_ord_xor_partial_ord -D clippy::from_over_into -D clippy::unwrap_used -D clippy::expect_used -D clippy::wildcard_imports -D clippy::mut_from_ref -D clippy::mutex_atomic -D clippy::mutex_integer -D clippy::redundant_allocation 2>&1 | grep -c 'error:')
    n_warnings=$(($n_warnings + $project_n_warnings - 1))
    project_code_count=$(tokei -t=Rust --output=json ${d} | jq -r '.Rust.code')
    code_count=$(($code_count + $project_code_count))
done

echo "Total Number of lines of Rust code: $code_count"
echo "Number of total warnings so far: $n_warnings"
