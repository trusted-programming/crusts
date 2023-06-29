#!/bin/sh

for d in */; do
  # Print the current working directory
  echo "Current directory: $(d)"
  # Run crusts
  (cd "${d}" && crusts)
done
