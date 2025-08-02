#!/bin/bash

# Command to run all lox scripts files for testing

# Exit immediately on errors.
set -e

LOX_DIRECTORY="lox_scripts"

if [ ! -d "$LOX_DIRECTORY" ]; then
  echo "Error: Directory '$LOX_DIRECTORY' not found."
  exit 1
fi

for file in "$LOX_DIRECTORY"/*.lox; do
  echo "*** Running: $file ***"

  cargo run -- "$file"

  echo "-------------------------"
done
