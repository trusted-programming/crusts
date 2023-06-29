#!/bin/sh

task() {
  # Print the current working directory
  echo "Started directory: $(d)"

  # Run crusts
  (cd "${d}" && crusts)
}

for d in */; do
  task &
done
