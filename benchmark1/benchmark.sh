#!/bin/sh

sucess_count=0
failed_count=0
total_count=0

for d in */; do
  (cd "${d}i" && crusts && cargo +nightly check)
  total_count=$((total_count + 1))
  if [[ $? -eq 0 ]]; then
    sucess_count=$((sucess_count + 1))
    else
    failed_count=$((failed_count + 1))
  fi
done

echo "Total $total_count times"
echo "Successfull $sucess_count times"
echo "Failed $failed_count times"