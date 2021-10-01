# Relational E-matching Artifact

[arXiv](https://arxiv.org/abs/2108.02290)

## Requirements

- Rust 1.55 or greater
- Python 3 with matplotlib

Both of these are in the provided container. 
Use `./docker.sh` to enter the container.

## Benchmarking Description

TODO 

## Instructions

The `./bench.sh` file is used to run the benchmarks. 
You can run three kinds of benchmarks.

- `./bench.sh short` is just for sanity checking,
  it only benches the smaller e-graphs and uses a 5s timeout.
  It takes around 2 minutes to run.

- `./bench.sh medium` runs the full suite, but with a 100s timeout and fewer samples.

- `./bench.sh full` runs the full suite with no timeout and the full 10 samples.
  The minimum time is taken for each benchmark across the 10 runs.

## Results Description

TODO 