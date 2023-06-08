#!/bin/sh

for d in */; do
  (cd "${d}i" && crusts && cargo +nightly check)
done
