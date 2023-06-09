#!/bin/sh

count=0
counter=0

for d in */; do
  (cd "$d" && crusts && cargo +nightly check)
  counter=$((counter + 1))
  if [[ $? -eq 0 ]]; then
    count=$((count + 1))
  fi
  echo "Total $counter times"
  echo "Failed $count times" 
done

echo "Total $counter times"
echo "Failed $count times"