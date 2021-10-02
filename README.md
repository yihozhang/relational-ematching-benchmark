# Relational E-matching Artifact

[arXiv](https://arxiv.org/abs/2108.02290)

## Requirements

- `make`
- Rust 1.55 or greater
- Python 3 with matplotlib

### Docker

All of the needed depedencies are provided in our docker container.
You can use `./docker.sh` to build and enter the container.
The first time you run `./docker.sh`, it may take a few minutes to build the image.
The script will leave you with a `bash` prompt, 
and you can run the benchmarking commands from there.

## Benchmarking Description

TODO 

## Instructions

The `make` command is used to run the benchmarks. 
You can run three kinds of benchmarks.

- `make short` is just for sanity checking,
  it only benches the smaller e-graphs and uses a 5s timeout.
  It takes around 2 minutes to run.

- `make medium` runs the full suite, but with a 100s timeout and fewer samples.

- `make full` runs the full suite with no timeout and the full 10 samples.
  The minimum time is taken for each benchmark across the 10 runs.

Additionally, `make submitted` will use the saved `out/benchmark.csv` 
to recreate the submitted results exactly.
This only does the calculations and plotting, 
`make full` will re-run the entire experiment.

## Results Description

TODO 