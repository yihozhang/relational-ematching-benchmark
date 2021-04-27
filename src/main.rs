use egg::{rewrite as rw, *};
use std::sync::mpsc;
use std::thread;
use std::time::*;
use std::*;

#[derive(Clone, Debug)]
pub struct Bench<L: Language> {
    name: String,
    start_expr: RecExpr<L>,
    rules: Vec<Rewrite<L, ()>>,
    bench_pats: Vec<Pattern<L>>,
}

mod math {
    use egg::{define_language, Id, Symbol};
    use ordered_float::NotNan;
    pub type Constant = NotNan<f64>;
    define_language! {
        pub enum Math {
            "+" = Add([Id; 2]),
            "-" = Sub([Id; 2]),
            "*" = Mul([Id; 2]),
            "/" = Div([Id; 2]),
            Constant(Constant),
            Symbol(Symbol),
        }
    }
}
use math::Math;

pub fn math_bench() -> Bench<Math> {
    let start_expr = "(+ (* y (+ x y)) (- (+ x 2) (+ x x)))".parse().unwrap();
    let rules = vec![
        rw!("comm-add";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
        rw!("comm-mul";  "(* ?a ?b)"        => "(* ?b ?a)"),
        // rw!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
        // rw!("assoc-mul"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),
        rw!("sub-canon"; "(- ?a ?b)" => "(+ ?a (* -1 ?b))"),
        rw!("zero-add"; "(+ ?a 0)" => "?a"),
        rw!("zero-mul"; "(* ?a 0)" => "0"),
        rw!("one-mul";  "(* ?a 1)" => "?a"),
        rw!("add-zero"; "?a" => "(+ ?a 0)"),
        rw!("mul-one";  "?a" => "(* ?a 1)"),
        rw!("cancel-sub"; "(- ?a ?a)" => "0"),
        rw!("distribute"; "(* ?a (+ ?b ?c))"        => "(+ (* ?a ?b) (* ?a ?c))"),
        rw!("factor"    ; "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),
    ];
    let bench_pats = vec![
        "(* ?a 1)",
        "(* ?a 0)",
        "(* ?a ?a)",
        "(* ?a ?b)",
        "(+ ?a ?b)",
        "(+ ?a (+ ?b ?c))",
        "(+ (+ ?a (+ 1 ?b)) (+ ?a (+ 1 ?c)))",
        "(+ (* ?a (+ 1 ?b)) (* ?a (+ 1 ?c)))",
    ]
    .iter()
    .map(|r| r.parse().unwrap())
    .collect();
    Bench {
        name: "math".into(),
        start_expr,
        rules,
        bench_pats,
    }
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

pub fn run_bench<L: Language + Sync + Send>(
    bench: Bench<L>,
    sizes: &Vec<usize>,
    strategies: &Vec<Strategy>,
    wtr: &mut csv::Writer<std::fs::File>,
) {
    let rules = bench.rules;
    let expr = bench.start_expr;
    let pats = bench.bench_pats;
    let mut egraph: EGraph<L, ()> = Default::default();
    for node_limit in sizes {
        egraph.strategy = Strategy::GenericJoin;
        let runner: egg::Runner<_, ()> = egg::Runner::default()
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
                        let t = thread::spawn(move || {
                            let time = std::time::Instant::now();
                            let res = pat.search(&egraph);
                            sender.send((time.elapsed().as_micros(), res)).unwrap_or_default()
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
                        time, result_size, repeat_time,
                    };
                    eprintln!("{:?}", record);
                    wtr.serialize(record).unwrap();
                    wtr.flush().unwrap();
                }
            }
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

fn main() {
    let opt = Opt::from_args();
    let benchmarks = opt.benchmarks.unwrap_or(vec!["math".into()]);
    let sizes = opt.sizes.unwrap_or(vec![100, 1000, 10000, 100000]);
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
    run_bench(math_bench(), &sizes, &strategies, &mut wtr);
}
