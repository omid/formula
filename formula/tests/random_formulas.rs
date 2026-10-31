//! Random formulas, valid or not, must never panic

use chrono::{DateTime, NaiveDate, NaiveTime};
use formula::{Expr, ExprError, Formula};
use std::panic;

/// Arguments of the random formulas, including references and values at the edges
const VALUES: &[&str] = &[
    "0",
    "1",
    "-1",
    "0.5",
    "2",
    "13",
    "60",
    "255",
    "1e308",
    "-1e308",
    "1e-308",
    "32767",
    "1048576",
    "'a'",
    "''",
    "'abc'",
    "'é'",
    "'1/2/2020'",
    "'12:30'",
    "\"x\"",
    "'\\n'",
    "TRUE",
    "FALSE",
    "{}",
    "{1, 'a'; TRUE, 2}",
    "A1",
    "B2",
    "C4",
    "Z99",
    "A1:C4",
    "C4:A1",
    "A:B",
    "2:3",
    "Sheet2!A1",
    "'My sheet'!A1:B2",
    "XFD1048576",
    "$A$1",
];

/// Characters that random edits insert, to break the formulas
const EDITS: &[char] = &[
    '(', ')', ',', ';', ':', '{', '}', '\'', '"', '!', '$', ' ', 'A', '1', '\\',
];

/// A table with values at the edges
fn table() -> Vec<Vec<Expr>> {
    vec![
        vec![1.into(), "a".into(), true.into()],
        vec![Expr::Null, f64::NAN.into(), f64::INFINITY.into()],
        vec![(-0.0).into(), 1e300.into(), ExprError::DivisionByZero.into()],
        vec![
            NaiveDate::MAX.into(),
            NaiveDate::MIN.into(),
            DateTime::<chrono::Utc>::MAX_UTC.into(),
            NaiveTime::MIN.into(),
            "a".repeat(40_000).into(),
            Expr::Array(vec![1.into(), Expr::Array(vec!["b".into()])]),
        ],
    ]
}

/// Builds random formulas from the functions of the grammar, with a fixed seed so failures repeat
struct Generator {
    state: u64,
    functions: Vec<String>,
}

impl Generator {
    fn new(seed: u64) -> Self {
        // WEBSERVICE would need the network
        let functions = include_str!("../src/formula.pest")
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .flat_map(|line| line.split("^\"").skip(1))
            .filter_map(|name| name.split('"').next())
            .filter(|name| *name != "WEBSERVICE")
            .map(str::to_string)
            .collect();
        Self { state: seed, functions }
    }

    /// xorshift64*
    fn below(&mut self, n: usize) -> usize {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        (self.state.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 33) as usize % n
    }

    fn expression(&mut self, depth: u32) -> String {
        if depth == 0 || self.below(4) == 0 {
            return VALUES[self.below(VALUES.len())].to_string();
        }

        let index = self.below(self.functions.len());
        let name = self.functions[index].clone();
        let args = (0..self.below(4))
            .map(|_| self.expression(depth - 1))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{name}({args})")
    }

    fn formula(&mut self) -> String {
        let mut formula = format!("={}", self.expression(4)).chars().collect::<Vec<_>>();

        // break some of them
        for _ in 0..self.below(4).saturating_sub(2) {
            let at = self.below(formula.len() + 1);
            if self.below(2) == 0 && at < formula.len() {
                formula.remove(at);
            } else {
                formula.insert(at, EDITS[self.below(EDITS.len())]);
            }
        }
        formula.into_iter().collect()
    }
}

#[test]
fn test_random_formulas_never_panic() {
    let table = table();
    let mut generator = Generator::new(0x5eed);
    assert!(generator.functions.len() > 100);

    let mut valid = 0;
    for _ in 0..20_000 {
        let formula = generator.formula();
        let result = panic::catch_unwind(|| {
            let Ok(parsed) = Formula::new(&formula) else {
                return false;
            };
            // evaluating never fails, errors are spreadsheet errors
            assert!(parsed.parse().is_ok());
            assert!(parsed.parse_with(&table).is_ok());
            true
        });
        let is_valid = result.unwrap_or_else(|_| panic!("{formula} panicked"));
        valid += usize::from(is_valid);
    }

    // enough of them are valid to evaluate the functions
    assert!(valid > 2_000, "only {valid} valid formulas");
}
