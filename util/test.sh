#!/bin/bash

lox=$1
echo "$lox ..."

# Use perl to grab the expected output, marked with "// expect: "
expected=$( perl -ne 'print "$1\n" if /\/\/ expect: (.*)/; print "error: $1\n" if /\/\/ expect runtime error: (.*)/;' < "$lox" )

# Run the interpreter (--quiet suppresses the "Compiling/Finished/Running" messages)
actual=$( cargo run --quiet -- $lox --simple-errors 2>&1 )

# Compare the expected output to the actual output
colordiff -u --label "$lox (expected)" <(echo "$expected") --label "$lox (actual)" <(echo "$actual")
