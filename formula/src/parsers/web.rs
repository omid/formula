use crate::{error::Error, Expr, Formula, Result, Rule};
use pest::iterators::Pair;
use urlencoding::encode;

impl Formula<'_> {
    pub(crate) fn parse_encodeurl(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let url = Self::get_formula(&mut args, &rule_name)?;

        let url = match url {
            Expr::String(url) => encode(&url).to_string(),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::String(url))
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
        todo!("Not implemented: {:?}", pair.as_rule())
    }

    pub(crate) fn parse_webservice(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let url = Self::get_formula(&mut args, &rule_name)?;

        let response = match url {
            Expr::String(url) =>
            /*reqwest::blocking::get(&url)
            .map_err(|_| Error::Parser(rule_name.clone()))?
            .text()
            .map_err(|_| Error::Parser(rule_name.clone()))?,*/
            {
                "hello".to_string()
            }
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::String(response))
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

        let formula = Formula::new("=WEBSERVICE('https://gist.githubusercontent.com/omid/3b4ec57e04dded792b14681b9aa1f724/raw/d6400777873f5a0ef7e916a46810a9a9af93dede/test')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("test".to_string()));
    }
}
