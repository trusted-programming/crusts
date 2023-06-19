#!/bin/sh

success_count=0
failed_count=0
total_count=0

for d in */; do
  total_count=$((total_count + 1))

  (cd "${d}i" && crusts && cargo +nightly check)

  if [ $? -eq 0 ]; then
    success_count=$((success_count + 1))
  else
    failed_count=$((failed_count + 1))
  fi
  echo "Total $total_count times"
  echo "Successful $success_count times"
  echo "Failed $failed_count times"
done