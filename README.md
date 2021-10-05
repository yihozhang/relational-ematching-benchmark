# Relational E-matching Artifact

This is the artifact for the
[POPL 2021 paper](https://popl22.sigplan.org/details/POPL-2022-popl-research-papers/35/Relational-E-Matching)
"Relational E-matching".
A copy can be found on 
[arXiv](https://arxiv.org/abs/2108.02290).

The paper introduces a new way to solve the e-matching problem 
using techniques from relational databases.
In particular, our implementation uses an algorithm called _generic join_.
The evaluation compares our approach (referred to in the paper and here as GJ)
with a traditional e-matching algorithm (referred to as EM).

This artifact aims to reproduce Figure 9 and Table 1.

## Benchmark Description

The empirical claims in this paper are based on a suite 
of experiments that compare the performance of the 
EM and GJ implementations.

The input to a benchmark is 
an e-graph (a set of terms) and 
a pattern to search for over those terms.
We run each benchmark three times:

- EM: Once with the traditional e-matching algorithm
- GJ0: Once with our algorithm
- GJ1: Again with GJ, using pre-built indexes.

Our benchmarking tool generates tables that look like this:

|index|  bench|       size|  gj|  em| TO|   total|    hmean|    gmean|     best|     medn|    worst |
| :-- |   :-- |       --: | --:| --:| --:|    --:|      --:|      --:|      --:|      --:|       --:|
|0|     lambda|       4142|  15|   3|  0|    1.69|      .84|     1.71|    13.62|     1.60|      .12 |
|1|     lambda|       4142|  18|   0|  0|    2.58|     2.99|     4.23|    39.17|     3.68|     1.10 |
|0|     lambda|      57454|  16|   2|  0|    2.60|      .95|     2.66|   136.54|     2.65|      .12 |
|1|     lambda|      57454|  18|   0|  0|    2.87|     3.33|     9.11|   406.70|     4.05|     1.03 |
|0|     lambda|     109493|  15|   3|  0|    1.66|     1.75|     3.11|   148.96|     2.03|      .65 |
|1|     lambda|     109493|  18|   0|  0|    1.70|     3.32|     7.46|   291.18|     4.10|     1.05 |
|0|     lambda|     213345|  15|   3|  0|    2.20|     1.55|     3.40|   304.33|     1.72|      .43 |
|1|     lambda|     213345|  18|   0|  0|    2.21|     2.96|     8.23|   501.12|     5.04|     1.04 |
|0|       math|       8205|  30|   2|  0|    5.49|      .64|     4.61|    66.54|     2.79|      .03 |
|1|       math|       8205|  30|   2|  0|    5.21|     2.93|     8.62|  1630.00|     5.48|      .62 |
|0|       math|      53286|  29|   3|  0|  311.23|     2.61|    13.50|    5.0e4|     3.62|      .74 |
|1|       math|      53286|  30|   2|  0|  318.95|     3.39|    29.60|    1.3e6|    30.72|      .74 |
|0|       math|     132080|  29|   3|  0|   96.55|     2.66|    15.18|    6.1e4|     4.02|      .60 |
|1|       math|     132080|  30|   2|  0|   97.84|     3.46|    34.16|    2.4e6|    68.71|      .75 |
|0|       math|     217396|  30|   2|  0|  119.82|     2.83|    18.34|    1.0e5|     3.91|      .72 |
|1|       math|     217396|  31|   1|  0|  119.73|     3.45|    41.35|    8.6e6|    80.84|      .76 |

This output corresponds to Table 1 in the paper. 

Each row corresponds to a comparison between EM and GJ0 or EM and GJ1, 
depending on the value in the index column. 
Each row aggregates across all patterns in that benchmark suite.
The columns indicate the following:

- index: whether we are comparing EM against GJ0 (which includes indexing time) or GJ1 (which excludes indexing time).
- bench: the benchmark suite
- size: the size of the e-graph in e-nodes
- gj: how many pattern GJ was faster on
- em: how many pattern EM was faster on
- TO: how many patterns EM timed out on (GJ should never timeout).
  *If this number is non-zero, that row looks worse for GJ (our tool) that it should, because an EM benchmark stopped "early" due to time out.*
- total: the ratio (total time spent in EM across all patterns) / (total time for GJ)
- the remaining columns show various summary stats across the speed-up ratios for each pattern.

## Requirements

- `make`
- Rust 1.55 or greater
- Python 3 with matplotlib

### Docker

All of the needed dependencies are provided in our docker container.
You can use `./docker.sh` to build and enter the container.
The first time you run `./docker.sh`, it may take a few minutes to build the image.
The script will leave you with a `bash` prompt, 
and you can run the benchmarking commands from there.

The `./docker.sh` script will automatically use [`podman`](https://podman.io/)
if available (likely the case if you are using Fedora), otherwise it will use `docker`.
You may need to do `sudo ./docker.sh` if you are using `docker`.

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
just over 14GB, so your machine 
(or VM if [running Docker on a Mac](https://docs.docker.com/desktop/mac/#resources)) 
should have 16GB or more. 
The `make short` variant will use less memory.

Running the benchmark will place the plot in the `out/` directory.

## Results Description

The `short` and `medium` variants will likely have non-zero numbers in the `TO`
column of the table.
This column is not in the paper because the `full` variant has no timeouts.
This number shows how many e-matching pattern timed out in that row's configuration,
our generic join implementation should never time out.
