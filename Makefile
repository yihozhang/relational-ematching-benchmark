
src = $(wildward src/*) Cargo.*
egg-bench = target/release/egg-bench
run = cargo run --release -- --verbose

$(egg-bench): $(src)
	cargo build --release

out/benchmark-short.csv: $(egg-bench)
	$(run) --filename=$@ --samples=1 --timeout=5 --sizes=10000,100000
out/benchmark-medium.csv: $(egg-bench)
	$(run) --filename=$@ --samples=1 --timeout=100
out/benchmark-full.csv: $(egg-bench)
	$(run) --filename=$@ --samples=1 --timeout=1000

# These are PHONY because we want to always run them to show the tables
.PHONY: short medium full submitted

submitted: out/benchmark.csv
	./compare.py $< --all-egraphs --plot=out/plot-submitted.pdf

short: out/benchmark-short.csv
	./compare.py $< --all-egraphs --plot=out/plot-short.pdf
medium: out/benchmark-medium.csv
	./compare.py $< --all-egraphs --plot=out/plot-medium.pdf
full: out/benchmark-full.csv
	./compare.py $< --all-egraphs --plot=out/plot-full.pdf