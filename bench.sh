#!/usr/bin/env bash

# The kind of benchmark to run.
# It should be one of `short`, `medium`, `full`.
# 
# See README.md
#
kind=$@

csv="out/benchmark-$kind.csv"
plot="plot-$kind.pdf"
run="cargo run --release -- --verbose --filename=$csv"

mkdir -p out

case "$kind" in
    short) 
        $run --samples=1 --timeout=5 --sizes=10000,100000
        ;;
    medium)  
        $run --samples=1 --timeout=10
        ;;
    full)  
        $run --samples=10 --timeout=1000
        ;;
    *) 
        echo "Unrecognized arg '$kind'. Use argument 'short', 'medium',  or 'full'"; 
        exit 1
        ;;
esac

./compare.py $csv --all-egraphs --plot=$plot