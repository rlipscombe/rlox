#!/bin/bash

lox=$1

# Use perl to grab the expected output, marked with "// expect: "
expected=$( perl -ne 'print "$1\n" if /\/\/ expect: (.*)/; print "runtime error: $1\n" if /\/\/ expect runtime error: (.*)/;' < "$lox" )

# Run the interpreter (--quiet suppresses the "Compiling/Finished/Running" messages)
actual=$( cargo run --quiet -- $lox --simple-errors 2>&1 )

# Compare the expected output to the actual output
colordiff -u --label "$lox" <(echo "$expected") <(echo "$actual")
