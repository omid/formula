use super::{Evaluator, Result};
use crate::{Expr, ExprError, Rule};
use pest::iterators::Pair;
use urlencoding::encode;

impl Evaluator<'_> {
    pub(crate) fn parse_encodeurl(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();

        match self.get_formula(&mut args)? {
            Expr::String(url) => Ok(Expr::String(encode(&url).into_owned())),
            _ => Err(ExprError::Value),
        }
    }

    // TODO a draft, `Formula::new` rejects it as not implemented yet
    // pub(crate) fn parse_filterxml(&self, pair: Pair<Rule>) -> Result<Expr> {
    //     let rule_name = format!("{:?}", &pair.as_rule());
    //     let mut args = pair.into_inner();
    //     let xml = self.get_formula(&mut args, &rule_name)?;
    //     let xpath = self.get_formula(&mut args, &rule_name)?;
    //
    //     let body = match (xml, xpath) {
    //         (Expr::String(xml), Expr::String(xpath)) => {
    //             let doc = parser::parse(&xml)?;
    //             evaluate_xpath(&doc.as_document(), &xpath)?.into_string()
    //         },
    //         _ => return Err(Error::Parser(rule_name).into()),
    //     };
    //     Ok(Expr::String(body))
    // }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn parse_webservice(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();

        match self.get_formula(&mut args)? {
            Expr::String(url) => Self::web_request(&url).map(Expr::String),
            _ => Err(ExprError::Value),
        }
    }

    /// Like Excel, failed requests are `#VALUE!`, also HTTP error responses (4xx, 5xx) instead of the error page
    #[cfg(not(target_arch = "wasm32"))]
    fn web_request(url: &str) -> Result<String> {
        reqwest::blocking::get(url)
            .and_then(reqwest::blocking::Response::error_for_status)
            .and_then(reqwest::blocking::Response::text)
            .map_err(|_| ExprError::Value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Formula;

    #[test]
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

        let formula = Formula::new("=FILTERXML('<a/>', '/a')");
        assert!(matches!(formula, Err(crate::error::Error::NotImplemented(_))));
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_parse_webservice() {
        let formula = format!("=WEBSERVICE('{}')", serve_once("200 OK", "test"));
        let value = Formula::new(&formula).unwrap().parse().unwrap();
        assert_eq!(value, Expr::String("test".to_string()));

        let formula = format!("=WEBSERVICE('{}')", serve_once("404 Not Found", "404: Not Found"));
        assert_eq!(
            Formula::new(&formula).unwrap().parse().unwrap(),
            Expr::Error(ExprError::Value)
        );

        let formula = format!("=WEBSERVICE('{}')", serve_once("500 Internal Server Error", "error"));
        assert_eq!(
            Formula::new(&formula).unwrap().parse().unwrap(),
            Expr::Error(ExprError::Value)
        );

        let formula = Formula::new("=WEBSERVICE('not a url')").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));
    }

    /// Answers one HTTP request on a local port with `status` and `body`, and returns its URL
    #[cfg(not(target_arch = "wasm32"))]
    fn serve_once(status: &'static str, body: &'static str) -> String {
        use std::{
            io::{BufRead, BufReader, Write},
            net::TcpListener,
        };

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();

            // read the request until the empty line that ends its headers
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap() > 2 {
                line.clear();
            }

            let len = body.len();
            write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}"
            )
            .unwrap();
        });
        url
    }
}
