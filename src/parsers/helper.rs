use crate::{error::Error, Expr, Formula, Rule};
use anyhow::Result;
use pest::iterators::Pairs;

impl Formula<'_> {
    pub(crate) fn get_formula(args: &mut Pairs<Rule>, rule_name: &str) -> Result<Expr> {
        Self::parse_pair(args.next().ok_or_else(|| Error::Parser(rule_name.to_owned()))?)
    }

    pub(crate) fn get_opt_formula_with_default(args: &mut Pairs<Rule>, default: Expr) -> Result<Expr> {
        args.next().map_or(Ok(default), Self::parse_pair)
    }
}
