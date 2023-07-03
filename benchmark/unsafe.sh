#!/bin/bash
for d in */; do

    # Use find to find all .rs files recursively in the given folder
    find "$d" -name "*.rs" -print0 | while IFS= read -r -d '' file; do
        # Use grep to check if the file contains the keyword "unsafe", if not skip the file
        if grep -q "unsafe" "$file"; then
            # Use sed to add comment above each line containing "unsafe"
            sed -i '/unsafe/ i\// SAFETY: machine generated unsafe code' "$file"
        fi
    done
done
