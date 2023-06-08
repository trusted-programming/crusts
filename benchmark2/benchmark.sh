#!/bin/sh

for d in */; do
  (cd "$d" && crusts -r && cargo +nightly check)
done
