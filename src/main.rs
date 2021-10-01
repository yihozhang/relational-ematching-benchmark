use egg::*;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc;
use std::thread;
use std::time::*;
use std::*;

mod lambda;
mod math;

#[derive(Clone, Debug)]
pub struct Bench<L: Language, A: Analysis<L> + 'static> {
    name: String,
    start_exprs: Vec<&'static str>,
    rules: Vec<Rewrite<L, A>>,
    bench_pats: Vec<Pattern<L>>,
}

fn parse_patterns<L: FromOp>(bench_name: &str) -> Vec<Pattern<L>> {
    let file = File::open("patterns.csv").unwrap();
    let reader = BufReader::new(file);
    let mut pats = vec![];
    for line in reader.lines().skip(1) {
        let line = line.unwrap();
        let line = line.trim();
        if !(line.is_empty() || line.starts_with('#')) {
            let fields: Vec<_> = line.split(",").map(|s| s.trim()).collect();
            if fields[0] == bench_name {
                let pat_string = fields.last().unwrap();
                pats.push(pat_string.parse().unwrap())
            }
        }
    }
    pats
}

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BenchRecord {
    benchmark: String,
    node_size: usize,
    class_size: usize,
    algo: String,
    pattern: String,
    time: String,
    result_size: usize,
    repeat_time: usize,
}

pub fn run_bench<L, A>(
    opt: &Opt,
    bench: Bench<L, A>,
    strategies: &[Strategy],
    wtr: &mut csv::Writer<std::fs::File>,
) where
    A: Analysis<L> + Default + Clone + Send + Sync,
    L: Language + FromOp + Sync + Send + Display,
    <A as egg::Analysis<L>>::Data: Send + Clone,
    <L as egg::Language>::Operator: Send + Sync,
{
    let rules = bench.rules;
    let pats = bench.bench_pats;
    let mut egraph: EGraph<L, A> = Default::default();
    for node_limit in &opt.sizes {
        egraph.strategy = Strategy::GenericJoin;
        let mut runner: Runner<L, A> = egg::Runner::default().with_egraph(egraph);
        for expr in &bench.start_exprs {
            runner = runner.with_expr(&expr.parse().unwrap());
        }

        let runner = runner
            .with_node_limit(*node_limit)
            .with_iter_limit(1000)
            .with_time_limit(std::time::Duration::from_secs(4000))
            .run(&rules);
        runner.print_report();
        egraph = runner.egraph;
        for pat in &pats {
            let mut em_time = None;
            let mut gj_time = None;
            for strategy in strategies {
                egraph.strategy = strategy.clone();
                let repeat = if strategy == &Strategy::GenericJoin {
                    2
                } else {
                    1
                };
                for repeat_time in 0..repeat {
                    // todo: add timeout
                    let (sender, receiver) = mpsc::channel();

                    let name = bench.name.clone();
                    {
                        let pat = pat.clone();
                        let egraph = egraph.clone();
                        thread::spawn(move || {
                            let time = std::time::Instant::now();
                            // let res = pat.search_with_limit(&egraph, 10_000_000);
                            let res = pat.search(&egraph);
                            // let res = pat.search_with_limit(&egraph, usize::MAX);
                            sender
                                .send((time.elapsed().as_micros(), res))
                                .unwrap_or_default()
                        });
                    }
                    let timeout = Duration::from_secs_f64(opt.timeout);
                    let (time, result_size) = receiver
                        .recv_timeout(timeout)
                        .map(|(time, res)| {
                            (
                                time.to_string(),
                                res.iter().map(|m| m.substs.len()).sum::<usize>(),
                            )
                        })
                        // timeouts are printed as negative
                        .unwrap_or((format!("-{}", timeout.as_micros()), 0));

                    match strategy {
                        Strategy::EMatch => em_time = Some(time.clone()),
                        Strategy::GenericJoin => gj_time = Some(time.clone()),
                    }

                    let record = BenchRecord {
                        benchmark: name,
                        node_size: egraph.total_number_of_nodes(),
                        class_size: egraph.number_of_classes(),
                        algo: format!("{:?}", egraph.strategy),
                        pattern: pat.pretty(usize::MAX),
                        time,
                        result_size,
                        repeat_time,
                    };
                    if opt.verbose {
                        eprintln!("{:?}", record);
                    }
                    wtr.serialize(record).unwrap();
                    wtr.flush().unwrap();
                }
            }

            if opt.verbose {
                if let (Some(gj), Some(em)) = (gj_time, em_time) {
                    if let (Ok(gj), Ok(em)) = (gj.parse::<f64>(), em.parse::<f64>()) {
                        let ratio = gj.abs() / em.abs();
                        if ratio > 1.0 {
                            println!("!!!!!!! BAD ratio: {}\n\n", ratio);
                        } else {
                            println!("        OK  ratio: {}", ratio);
                        }
                    }
                }
            }
        }
        if let Some(StopReason::Saturated) = runner.stop_reason {
            break;
        }
    }
}

use structopt::StructOpt;
#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(short, long, value_delimiter = ",", default_value = "math,lambda")]
    benchmarks: Vec<String>,
    #[structopt(
        short,
        long,
        value_delimiter = ",",
        default_value = "10000,100000,200000,300000"
    )]
    sizes: Vec<usize>,
    #[structopt(short, long, default_value = "out/benchmark.csv")]
    filename: String,
    #[structopt(long, default_value = "all")]
    strategy: String,
    #[structopt(long, default_value = "1")]
    samples: usize,
    #[structopt(long, default_value = "60")]
    timeout: f64,
    #[structopt(long)]
    verbose: bool,
}

fn math(opt: &Opt, strategies: &[Strategy], wtr: &mut csv::Writer<std::fs::File>) {
    run_bench(opt, math::math_bench(), strategies, wtr)
}

fn lambda(opt: &Opt, strategies: &[Strategy], wtr: &mut csv::Writer<std::fs::File>) {
    run_bench(opt, lambda::lambda_bench(), strategies, wtr)
}

fn main() {
    let start = Instant::now();
    let _ = env_logger::init();
    let opt = Opt::from_args();
    let strategies = match opt.strategy.as_str() {
        "all" => vec![Strategy::GenericJoin, Strategy::EMatch],
        "gj" => vec![Strategy::GenericJoin],
        "em" => vec![Strategy::EMatch],
        _ => panic!("strategy should be one of all, gj, or em"),
    };
    let out = std::fs::File::create(&opt.filename).unwrap();
    let mut wtr = csv::Writer::from_writer(out);
    let mut bench_collection: collections::HashMap<String, fn(_, _, &mut _)> = Default::default();
    bench_collection.insert("math".into(), math);
    bench_collection.insert("lambda".into(), lambda);
    for _ in 0..opt.samples {
        for bench in &opt.benchmarks {
            let bench_fn = &bench_collection[&bench.clone()];
            bench_fn(&opt, &strategies, &mut wtr);
        }
    }

    println!("Benchmark took {:?}", start.elapsed())
}
