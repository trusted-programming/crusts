#!/bin/sh

# success_count=0
# failed_count=0
# total_count=0

# for d in */; do
#   total_count=$((total_count + 1))

#   (cd "${d}" && crusts && cargo +nightly check)

#   if [ $? -eq 0 ]; then
#     success_count=$((success_count + 1))
#   else
#     failed_count=$((failed_count + 1))
#   fi
#   echo "Total $total_count times"
#   echo "Successful $success_count times"
#   echo "Failed $failed_count times"
# done

#!/bin/sh

success_count=0
failed_count=0
total_count=0
n_warnings=0
code_count=0
success_folders=""
failed_folders=""

for d in */; do
  total_count=$((total_count + 1))

  (cd "${d}" && crusts && cargo +nightly check)

  if [ $? -eq 0 ]; then
    success_count=$((success_count + 1))
    success_folders="${success_folders}${d%/}\n"
    project_n_warnings=$(cd "${d}" && cargo clippy --message-format=json 2>&1 | grep -c 'warning:')
    n_warnings=$(($n_warnings + $project_n_warnings - 1))
    project_code_count=$(tokei -t=Rust --output=json ${d} | jq -r '.Rust.code')
    code_count=$(($code_count + $project_code_count))

  else
    failed_count=$((failed_count + 1))
    failed_folders="${failed_folders}${d%/}\n"
  fi

  echo -e "Success folders:\n$success_folders"
  echo -e "Failed folders:\n$failed_folders"
  echo "Total $total_count times"
  echo "Successful $success_count times"
  echo "Failed $failed_count times"
  echo "Total Number of lines of Rust code: $code_count"
  echo "Number of total warnings so far: $n_warnings"
done
