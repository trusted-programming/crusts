#!/bin/sh

task() {
  (cd "${d}" && crusts)
}

for d in */; do
  task &
done
