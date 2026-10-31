use formula::{Cells, Expr, Formula, Result, Size};

/// A workbook with named sheets, formulas belong to its first sheet
struct Workbook {
    sheets: Vec<(String, Vec<Vec<Expr>>)>,
}

impl Workbook {
    /// Like Excel, sheet names are case-insensitive
    fn sheet(&self, name: Option<&str>) -> Option<&[Vec<Expr>]> {
        let sheet = match name {
            None => self.sheets.first(),
            Some(name) => self.sheets.iter().find(|(sheet, _)| sheet.eq_ignore_ascii_case(name)),
        };
        sheet.map(|(_, rows)| rows.as_slice())
    }
}

impl Cells for Workbook {
    fn size(&self, sheet: Option<&str>) -> Option<Size> {
        self.sheet(sheet)?.size(None)
    }

    fn cell(&self, sheet: Option<&str>, row: u32, column: u32) -> Option<Expr> {
        self.sheet(sheet)?.cell(None, row, column)
    }
}

fn main() -> Result<()> {
    // rows of values, the second pear's count is empty
    let fruits = vec![
        vec![Expr::from("Apples"), Expr::from(1.5), Expr::from(4)],
        vec![Expr::from("Pears"), Expr::from(2), Expr::Null],
    ];
    let formula = Formula::new("=SUM(C:C)")?;
    assert_eq!(formula.parse_with(&fruits)?, Expr::Number(4.0));

    // the same formula with other data
    let formula = Formula::new("=F.MUL(B1, C1)")?;
    assert_eq!(formula.parse_with(&fruits)?, Expr::Number(6.0));
    assert_eq!(
        formula.parse_with(&vec![vec![Expr::from("Plums"), Expr::from(3), Expr::from(5)]])?,
        Expr::Number(15.0)
    );

    // several sheets
    let workbook = Workbook {
        sheets: vec![
            ("Fruits".to_string(), fruits),
            ("Discounts".to_string(), vec![vec![Expr::from(0.5)]]),
        ],
    };
    let formula = Formula::new("=F.MUL(B1, Discounts!A1)")?;
    assert_eq!(formula.parse_with(&workbook)?, Expr::Number(0.75));
    let formula = Formula::new("=TEXTJOIN(', ', TRUE, fruits!A:A)")?;
    assert_eq!(formula.parse_with(&workbook)?, Expr::from("Apples, Pears"));
    Ok(())
}
