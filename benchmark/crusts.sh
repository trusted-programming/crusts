#!/bin/sh

for d in */; do
  (cd "${d}" && crusts)
done
