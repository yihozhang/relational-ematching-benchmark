use egg::*;
use std::sync::mpsc;
use std::thread;
use std::time::*;
use std::*;

mod math;
mod lambda;

#[derive(Clone, Debug)]
pub struct Bench<L: Language, A: Analysis<L> + 'static> {
    name: String,
    start_expr: RecExpr<L>,
    rules: Vec<Rewrite<L, A>>,
    bench_pats: Vec<Pattern<L>>,
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
    bench: Bench<L, A>,
    sizes: &Vec<usize>,
    strategies: &Vec<Strategy>,
    wtr: &mut csv::Writer<std::fs::File>,
) where
    A: Analysis<L> + Default + Clone + Send + Sync,
    L: Language + Sync + Send,
    <A as egg::Analysis<L>>::Data: Send + Clone,
    <L as egg::Language>::Operator: Send + Sync,
{
    let rules = bench.rules;
    let expr = bench.start_expr;
    let pats = bench.bench_pats;
    let mut egraph: EGraph<L, A> = Default::default();
    for node_limit in sizes {
        egraph.strategy = Strategy::GenericJoin;
        let runner: Runner<L, A> = egg::Runner::default()
            .with_egraph(egraph)
            .with_expr(&expr)
            .with_node_limit(*node_limit)
            .with_iter_limit(1000)
            .with_time_limit(std::time::Duration::from_secs(4000))
            .run(&rules);
        runner.print_report();
        egraph = runner.egraph;
        for pat in &pats {
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
                            let res = pat.search_with_limit(&egraph, 10_000_000);
                            // let res = pat.search_with_limit(&egraph, usize::MAX);
                            sender
                                .send((time.elapsed().as_micros(), res))
                                .unwrap_or_default()
                        });
                    }
                    let (time, result_size) = receiver
                        .recv_timeout(Duration::from_secs(5))
                        .map(|(time, res)| {
                            (
                                time.to_string(),
                                res.iter().map(|m| m.substs.len()).sum::<usize>(),
                            )
                        })
                        .unwrap_or(("TO".into(), 0));
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
                    eprintln!("{:?}", record);
                    wtr.serialize(record).unwrap();
                    wtr.flush().unwrap();
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
struct Opt {
    #[structopt(short, long)]
    benchmarks: Option<Vec<String>>,
    #[structopt(short, long)]
    sizes: Option<Vec<usize>>,
    #[structopt(short, long)]
    filename: Option<String>,
    #[structopt(long)]
    strategy: Option<String>,
}

fn math(
    sizes: &Vec<usize>,
    strategies: &Vec<Strategy>,
    wtr: &mut csv::Writer<std::fs::File>,
) {
    run_bench(math::math_bench(), sizes, strategies, wtr)
}

fn lambda1(
    sizes: &Vec<usize>,
    strategies: &Vec<Strategy>,
    wtr: &mut csv::Writer<std::fs::File>,
) {
    run_bench(lambda::lambda_bench1(), sizes, strategies, wtr)
}

fn lambda2(
    sizes: &Vec<usize>,
    strategies: &Vec<Strategy>,
    wtr: &mut csv::Writer<std::fs::File>,
) {
    run_bench(lambda::lambda_bench2(), sizes, strategies, wtr)
}

fn main() {
    let opt = Opt::from_args();
    let benchs = opt.benchmarks.unwrap_or(vec!["math".into()]);
    let sizes = opt.sizes.unwrap_or(vec![100, 1000, 10000, 100000, 200000, 300000]);
    let filename = opt.filename.unwrap_or("out/benchmark.csv".into());
    let strategies = opt.strategy.unwrap_or("all".into());
    let strategies = match strategies.as_str() {
        "all" => vec![Strategy::GenericJoin, Strategy::EMatch],
        "gj" => vec![Strategy::GenericJoin],
        "em" => vec![Strategy::EMatch],
        _ => panic!("strategy should be one of all, gj, or em"),
    };
    let out = std::fs::File::create(&filename).unwrap();
    let mut wtr = csv::Writer::from_writer(out);
    let mut bench_collection: collections::HashMap<String, fn(_, _, &mut _)> = Default::default();
    bench_collection.insert("math".into(), math);
    bench_collection.insert("lambda1".into(), lambda1);
    bench_collection.insert("lambda2".into(), lambda2);
    for bench in benchs {
        let bench_fn = &bench_collection[&bench];
        bench_fn(&sizes, &strategies, &mut wtr);
    }
}
