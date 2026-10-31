//! Benchmarks of parsing and evaluating formulas, run them with `make bench`

use criterion::{Criterion, criterion_group, criterion_main};
use formula::{Cells, Expr, Formula, Size};
use std::{hint::black_box, time::Duration};

/// `rows` rows of `columns` numbers
fn numbers(rows: usize, columns: usize) -> Vec<Vec<Expr>> {
    (0..rows)
        .map(|row| {
            (0..columns)
                .map(|column| Expr::from((row * columns + column) as f64))
                .collect()
        })
        .collect()
}

/// `rows` rows of a 10 character text
fn texts(rows: usize) -> Vec<Vec<Expr>> {
    (0..rows)
        .map(|row| vec![Expr::from(format!("text {row:05}"))])
        .collect()
}

/// Rows of a number, a text, an empty cell and a boolean
fn mixed(rows: usize) -> Vec<Vec<Expr>> {
    (0..rows)
        .map(|row| vec![Expr::from(row as f64), Expr::from("text"), Expr::Null, Expr::from(true)])
        .collect()
}

/// Sheets with one number each
struct Sheets;

impl Cells for Sheets {
    fn size(&self, sheet: Option<&str>) -> Option<Size> {
        matches!(sheet, None | Some("My sheet" | "Sheet2")).then_some(Size { rows: 1, columns: 1 })
    }

    fn cell(&self, _sheet: Option<&str>, _row: u32, _column: u32) -> Option<Expr> {
        Some(Expr::Number(1.0))
    }
}

fn evaluate(c: &mut Criterion, name: &str, formula: &str, cells: &(impl Cells + ?Sized)) {
    let formula = Formula::new(formula).unwrap();
    c.bench_function(name, |b| b.iter(|| formula.parse_with(black_box(cells))));
}

fn parse(c: &mut Criterion) {
    let formula = "=IF(F.GT(SUM(1, 2, 3), 5), CONCAT(UPPER(LEFT('hello world', 5)), ' ', TRIM('  a  b  ')), \
                   DATE(2020, 1, 1))";
    c.bench_function("parse/new", |b| b.iter(|| Formula::new(black_box(formula))));
    evaluate(c, "parse/evaluate", formula, &Vec::new());

    let escapes = format!("='{}'", r"\u00e9".repeat(1000));
    c.bench_function("parse/escapes", |b| {
        b.iter(|| Formula::new(black_box(&escapes)).unwrap().parse());
    });
}

fn ranges(c: &mut Criterion) {
    let numbers_100k = numbers(100_000, 1);
    let booleans = vec![vec![Expr::from(true)]; 100_000];
    let texts_100k = texts(100_000);

    evaluate(c, "range/sum_column", "=SUM(A:A)", &numbers_100k);
    evaluate(c, "range/sum_mixed", "=SUM(A:D)", &mixed(25_000));
    evaluate(c, "range/and_column", "=AND(A:A)", &booleans);
    evaluate(c, "range/concat", "=CONCAT(A1:A3000)", &texts_100k);
    evaluate(c, "range/textjoin", "=TEXTJOIN(',', TRUE, A1:A2900)", &texts_100k);
    evaluate(c, "range/concat_too_long", "=CONCAT(A:A)", &texts_100k);
    evaluate(c, "range/value", "=A1:J10000", &numbers(10_000, 10));
}

fn texts_functions(c: &mut Criterion) {
    let long = vec![vec![
        Expr::from("lorem ipsum ".repeat(1000)),
        Expr::from("word    ".repeat(1000)),
    ]];

    evaluate(c, "text/left", "=LEFT(A1, 6000)", &long);
    evaluate(c, "text/right", "=RIGHT(A1, 6000)", &long);
    evaluate(c, "text/mid", "=MID(A1, 3000, 6000)", &long);
    evaluate(c, "text/replace", "=REPLACE(A1, 3000, 6000, 'x')", &long);
    evaluate(c, "text/trim", "=TRIM(B1)", &long);
}

fn references(c: &mut Criterion) {
    evaluate(c, "reference/cells", "=F.ADD(A1, B2)", &numbers(10, 10));
    evaluate(c, "reference/sheets", "=F.ADD('My sheet'!A1, Sheet2!A1)", &Sheets);
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(3));
    targets = parse, ranges, texts_functions, references
}
criterion_main!(benches);
