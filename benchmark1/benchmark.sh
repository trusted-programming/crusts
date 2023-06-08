#!/bin/sh

for d in */; do
  (cd "$d" && crusts && cargo +nightly check)
done
