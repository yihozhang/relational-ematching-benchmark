# Relational E-matching Artifact

[Link to arXiv.](https://arxiv.org/abs/2108.02290)

This artifact aims to reproduce Figure 9 and Table 1.

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

The `./docker.sh` script will automatically use [`podman`](https://podman.io/)
if available (likely the case if you are using Fedora), otherwise it will use `docker`.
You may need to do `sudo ./docker.sh` if you are using `docker`.

## Benchmarking Description

TODO 

## Instructions

The `make` command is used to run the benchmarks. 
You can run three kinds of benchmarks.

- `make short` is just for sanity checking,
  it only benches the smaller e-graphs and uses a 5s timeout.
  It takes around 2 minutes to run.

- `make medium` runs the full suite, but with a 100s timeout and fewer samples.
  This takes about 15-20 minutes to run.

- `make full` runs the full suite with no timeout and 5 samples.
  The minimum time is taken for each benchmark across the 5 runs.
  This takes 2-4 hours to run.

Additionally, `make submitted` will use the saved `out/benchmark.csv` 
to recreate the submitted results exactly.
This only does the calculations and plotting, 
`make full` will re-run the entire experiment.

The first time you run `make`, 
it will have to build the project, which will take a couple minutes.
This will be cached for future invocations of `make`.

GNU `time` reports that the maximum resident memory used by the benchmark is
just over 14GB, so your machine (or VM if running Docker on a Mac) should have
16GB or more. 
The `make short` variant will use less memory.

Running the benchmark will place the plot in the `out/` directory.

## Results Description

The `short` and `medium` variants will likely have non-zero numbers in the `TO`
column of the table.
This column is not in the paper because the `full` variant has no timeouts.
This number shows how many e-matching pattern timed out in that row's configuration,
our generic join implementation should never time out.
If this number is non-zero, that row looks worse for GJ (our tool) that it should,
because an EM benchmark stopped "early" due to time out.