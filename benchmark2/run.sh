#!/bin/sh

for d in */; do
  (cd "$d" && rm -rf target)
done
