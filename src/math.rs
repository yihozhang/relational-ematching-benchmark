
use egg::{define_language, Id, Symbol, rewrite as rw};
use crate::*;
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

pub fn math_bench() -> Bench<Math, ()> {
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
        "(* (+ ?a ?b) ?c)",
        "(+ (+ ?a (+ 1 ?b)) (+ ?a (+ 1 ?c)))",
        "(+ (* ?a (+ 1 ?b)) (* ?a (+ 1 ?c)))",
        "(+ (* ?a ?b) (* ?a ?c))",
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
