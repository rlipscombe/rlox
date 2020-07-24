#!/bin/bash

lox=$1

# Use perl to grab the expected output, marked with "// expect: "
expected=$( perl -ne '/\/\/ expect: (.*)/ && print $1 . "\n"' < $lox )
# Run the interpreter (--quiet suppresses the "Compiling/Finished/Running" messages)
actual=$( cargo run --quiet $lox 2>&1 )

# Compare the expected output to the actual output
colordiff -u <(echo "$expected") <(echo "$actual")
