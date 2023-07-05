#!/bin/bash

for d in */; do
    # Print the current working directory
    echo "Current directory: ${d}"

    # run clippy fix
    (cd "${d}" && cargo +nightly-2023-06-02 clippy --fix --allow-dirty --allow-no-vcs --allow-stagedcd)
done
