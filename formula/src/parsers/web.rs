use super::{helper::parser_error, Result};
use crate::{error::Error, Expr, Formula, Rule};
use pest::iterators::Pair;
use urlencoding::encode;

impl Formula<'_> {
    pub(crate) fn parse_encodeurl(pair: Pair<Rule>) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();

        match Self::get_formula(&mut args, rule)? {
            Expr::String(url) => Ok(Expr::String(encode(&url).into_owned())),
            _ => Err(parser_error(rule)),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn parse_filterxml(pair: Pair<Rule>) -> Result<Expr> {
        // let rule_name = format!("{:?}", &pair.as_rule());
        // let mut args = pair.into_inner();
        // let xml = Self::get_formula(&mut args, &rule_name)?;
        // let xpath = Self::get_formula(&mut args, &rule_name)?;
        //
        // let body = match (xml, xpath) {
        //     (Expr::String(xml), Expr::String(xpath)) => {
        //         let doc = parser::parse(&xml)?;
        //         evaluate_xpath(&doc.as_document(), &xpath)?.into_string()
        //     },
        //     _ => return Err(Error::Parser(rule_name).into()),
        // };
        // Ok(Expr::String(body))
        Err(Error::NotImplemented(format!("{:?}", pair.as_rule())).into())
    }

    pub(crate) fn parse_webservice(pair: Pair<Rule>) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();

        match Self::get_formula(&mut args, rule)? {
            Expr::String(url) => Self::web_request(&url, rule).map(Expr::String),
            _ => Err(parser_error(rule)),
        }
    }

    /// HTTP error responses (4xx, 5xx) are errors too, instead of returning the error page as the value
    #[cfg(not(target_arch = "wasm32"))]
    fn web_request(url: &str, rule: Rule) -> Result<String> {
        reqwest::blocking::get(url)
            .and_then(reqwest::blocking::Response::error_for_status)
            .and_then(reqwest::blocking::Response::text)
            .map_err(|_| parser_error(rule))
    }

    #[cfg(target_arch = "wasm32")]
    fn web_request(_url: &str, rule: Rule) -> Result<String> {
        Err(Error::NotImplemented(format!("{rule:?}")).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Formula;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_parse_web_types() {
        let formula =
            Formula::new("=ENCODEURL('http://contoso.sharepoint.com/Finance/Profit and Loss Statement.xlsx')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(
            value,
            Expr::String(
                "http%3A%2F%2Fcontoso.sharepoint.com%2FFinance%2FProfit%20and%20Loss%20Statement.xlsx".to_string()
            )
        );

        // let formula = Formula::new("=FILTERXML(WEBSERVICE('http://example.com'), '/html/head/title')").unwrap();
        // let value = formula.parse().unwrap();
        // assert_eq!(value, Expr::String("test".to_string()));

        let formula = Formula::new("=WEBSERVICE('https://gist.githubusercontent.com/omid/e93e7f5a157b293b92c087c0e3e8bc61/raw/e794820ce82b6aa2a989ba64bf713bf27090e430/test')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("test".to_string()));

        let formula = Formula::new("=FILTERXML('<a/>', '/a')").unwrap();
        assert!(matches!(formula.parse(), Err(Error::NotImplemented(_))));
    }
}
