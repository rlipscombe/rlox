#!/bin/bash

test="$1"
find "$test" -name '*.lox' -exec $(dirname $0)/test.sh {} \;
