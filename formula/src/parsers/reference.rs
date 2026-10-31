use super::{Evaluator, Result};
use crate::{Expr, ExprError, Rule};
use pest::iterators::Pair;

/// Excel's last column, `XFD`
const MAX_COLUMN: u32 = 16_384;
/// Excel's last row
const MAX_ROW: u32 = 1_048_576;
/// The most cells a range can read, one whole Excel column
const MAX_RANGE_CELLS: u64 = 1_048_576;

/// A reference like `Sheet2!B3:A1`, with its corners in order
pub(crate) struct Reference {
    sheet: Option<String>,
    /// The first and last row, `None` for whole columns like `A:C`
    rows: Option<(u32, u32)>,
    /// The first and last column, `None` for whole rows like `2:3`
    columns: Option<(u32, u32)>,
    /// A single cell like `A1`, instead of a range like `A1:A1`
    is_cell: bool,
}

impl Reference {
    /// `None` if the reference is outside of an Excel sheet, like `XFE1` or `A1048577`
    pub(crate) fn new(pair: &Pair<Rule>) -> Option<Self> {
        let mut sheet = None;
        let mut rows = Vec::with_capacity(2);
        let mut columns = Vec::with_capacity(2);
        for part in pair.clone().into_inner() {
            match part.as_rule() {
                Rule::sheet_name => sheet = Some(part.as_str().to_string()),
                Rule::quoted_sheet_name => sheet = Some(part.as_str().replace("''", "'")),
                Rule::column => columns.push(Self::column_number(part.as_str())?),
                Rule::row => rows.push(Self::row_number(part.as_str())?),
                _ => {}
            }
        }

        Some(Self {
            sheet,
            is_cell: rows.len() == 1 && columns.len() == 1,
            rows: Self::span(&rows),
            columns: Self::span(&columns),
        })
    }

    /// `A` is 1, `Z` is 26, `AA` is 27…
    fn column_number(column: &str) -> Option<u32> {
        let number = column.trim_start_matches('$').bytes().fold(0, |number, letter| {
            number * 26 + u32::from(letter.to_ascii_uppercase() - b'A' + 1)
        });
        (number <= MAX_COLUMN).then_some(number)
    }

    fn row_number(row: &str) -> Option<u32> {
        row.trim_start_matches('$').parse().ok().filter(|row| *row <= MAX_ROW)
    }

    /// The first and last of one or two corners
    fn span(corners: &[u32]) -> Option<(u32, u32)> {
        match *corners {
            [corner] => Some((corner, corner)),
            [start, end] => Some((start.min(end), start.max(end))),
            _ => None,
        }
    }
}

impl Evaluator<'_> {
    /// A cell is its value, a range is its rows like array literals: `A1:B2` is like `{A1, B1; A2, B2}`
    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn parse_reference(&self, pair: Pair<Rule>) -> Result<Expr> {
        // `Formula::new` rejected the references outside of a sheet
        let reference = Reference::new(&pair).ok_or(ExprError::Reference)?;
        let sheet = reference.sheet.as_deref();
        let size = self.cells.size(sheet).ok_or(ExprError::Reference)?;
        let read = |row, column| self.cells.cell(sheet, row, column).unwrap_or(Expr::Null);

        // whole columns and rows only cover the cells with data
        let (first_row, last_row) = reference.rows.unwrap_or((1, size.rows));
        let (first_column, last_column) = reference.columns.unwrap_or((1, size.columns));
        if reference.is_cell {
            return Ok(read(first_row, first_column));
        }

        let height = (u64::from(last_row) + 1).saturating_sub(u64::from(first_row));
        let width = (u64::from(last_column) + 1).saturating_sub(u64::from(first_column));
        if height * width > MAX_RANGE_CELLS {
            return Err(ExprError::Number);
        }

        let mut rows = (first_row..=last_row)
            .map(|row| Expr::Array((first_column..=last_column).map(|column| read(row, column)).collect()))
            .collect::<Vec<_>>();

        // like array literals, a single row is flat
        if rows.len() == 1 {
            Ok(rows.remove(0))
        } else {
            Ok(Expr::Array(rows))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Cells, Expr, ExprError, Formula, Size, error::Error};

    /// 3×3 cells, B2 is empty and C3 is `#DIV/0!`
    fn table() -> Vec<Vec<Expr>> {
        vec![
            vec![1.into(), 2.into(), 3.into()],
            vec![4.into(), Expr::Null, 6.into()],
            vec![7.into(), 8.into(), ExprError::DivisionByZero.into()],
        ]
    }

    fn parse(formula: &str) -> Expr {
        Formula::new(formula).unwrap().parse_with(&table()).unwrap()
    }

    fn row(values: &[Expr]) -> Expr {
        Expr::Array(values.to_vec())
    }

    /// Sheets named after their only cell's value
    struct Sheets;

    impl Cells for Sheets {
        fn size(&self, sheet: Option<&str>) -> Option<Size> {
            matches!(sheet, Some("Sheet2" | "My sheet" | "It's")).then_some(Size { rows: 1, columns: 1 })
        }

        fn cell(&self, sheet: Option<&str>, _row: u32, _column: u32) -> Option<Expr> {
            sheet.map(Expr::from)
        }
    }

    /// Claims more rows than a range can read
    struct Huge;

    impl Cells for Huge {
        fn size(&self, _sheet: Option<&str>) -> Option<Size> {
            Some(Size {
                rows: u32::MAX,
                columns: 1,
            })
        }

        fn cell(&self, _sheet: Option<&str>, _row: u32, _column: u32) -> Option<Expr> {
            Some(Expr::Number(1.0))
        }
    }

    #[test]
    fn test_cells() {
        assert_eq!(parse("=A1"), Expr::Number(1.0));
        assert_eq!(parse("=$C$1"), Expr::Number(3.0));
        assert_eq!(parse("=c$1"), Expr::Number(3.0));
        assert_eq!(parse("=F.ADD(A1, C2)"), Expr::Number(7.0));
        assert_eq!(parse("={A1, B1}"), row(&[1.into(), 2.into()]));

        // empty, and outside of the data
        assert_eq!(parse("=B2"), Expr::Null);
        assert_eq!(parse("=XFD1048576"), Expr::Null);

        // errors in cells propagate
        assert_eq!(parse("=C3"), Expr::Error(ExprError::DivisionByZero));
        assert_eq!(parse("=F.ADD(C3, 1)"), Expr::Error(ExprError::DivisionByZero));
        assert_eq!(parse("=IFERROR(C3, 0)"), Expr::Number(0.0));

        // a range where a single value is needed
        assert_eq!(parse("=F.ADD(A1:A2, 1)"), Expr::Error(ExprError::Value));

        // without data, cells are empty
        let formula = Formula::new("=A1").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Null);

        // the same formula with other data
        assert_eq!(formula.parse_with(&table()).unwrap(), Expr::Number(1.0));
        assert_eq!(
            formula.parse_with(&vec![vec![Expr::from("x")]]).unwrap(),
            Expr::from("x")
        );
    }

    #[test]
    fn test_ranges() {
        // a single row is flat, like array literals
        assert_eq!(parse("=A1:C1"), row(&[1.into(), 2.into(), 3.into()]));
        assert_eq!(parse("=A1:A1"), row(&[1.into()]));

        // corners in any order
        let a1_b2 = row(&[row(&[1.into(), 2.into()]), row(&[4.into(), Expr::Null])]);
        assert_eq!(parse("=A1:B2"), a1_b2);
        assert_eq!(parse("=B2:A1"), a1_b2);
        assert_eq!(parse("=A2:B1"), a1_b2);

        // errors stay in their cells
        assert_eq!(
            parse("=B3:C3"),
            row(&[8.into(), Expr::Error(ExprError::DivisionByZero)])
        );

        // whole columns and rows only cover the cells with data
        assert_eq!(
            parse("=A:A"),
            row(&[row(&[1.into()]), row(&[4.into()]), row(&[7.into()])])
        );
        assert_eq!(parse("=B:A"), parse("=A1:B3"));
        assert_eq!(parse("=2:2"), row(&[4.into(), Expr::Null, 6.into()]));
        assert_eq!(parse("=$3:2"), parse("=A2:C3"));
        assert_eq!(
            parse("=D:D"),
            row(&[row(&[Expr::Null]), row(&[Expr::Null]), row(&[Expr::Null])])
        );
        assert_eq!(
            parse("=4:5"),
            row(&[
                row(&[Expr::Null, Expr::Null, Expr::Null]),
                row(&[Expr::Null, Expr::Null, Expr::Null])
            ])
        );
        let formula = Formula::new("=A:A").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Array(vec![]));

        // too many cells
        assert_eq!(parse("=A1:XFD1048576"), Expr::Error(ExprError::Number));
        let formula = Formula::new("=A:A").unwrap();
        assert_eq!(formula.parse_with(&Huge).unwrap(), Expr::Error(ExprError::Number));
    }

    #[test]
    fn test_sheets() {
        let parse = |formula| Formula::new(formula).unwrap().parse_with(&Sheets).unwrap();

        assert_eq!(parse("=Sheet2!A1"), Expr::from("Sheet2"));
        assert_eq!(parse("='My sheet'!$A$1"), Expr::from("My sheet"));
        assert_eq!(parse("='It''s'!A1:A1"), row(&[Expr::from("It's")]));
        assert_eq!(parse("=Sheet3!A1"), Expr::Error(ExprError::Reference));
        assert_eq!(parse("=Sheet3!A:A"), Expr::Error(ExprError::Reference));

        // the formula's own sheet doesn't exist here
        assert_eq!(parse("=A1"), Expr::Error(ExprError::Reference));

        // rows of values have no sheets
        assert_eq!(super::tests::parse("=Sheet2!A1"), Expr::Error(ExprError::Reference));
    }

    #[test]
    fn test_reference_grammar() {
        // functions whose names look like cells
        assert_eq!(parse("=LOG10(100)"), Expr::Number(2.0));
        assert_eq!(parse("=LOG10 (100)"), Expr::Number(2.0));
        assert_eq!(parse("=DAY(DATE(2020,1,2))"), Expr::Number(2.0));
        assert_eq!(parse("=ATAN2(1, 1)"), Expr::Number(std::f64::consts::FRAC_PI_4));
        assert_eq!(parse("=LOG10"), Expr::Null);

        // other values that are not references
        assert_eq!(parse("=TRUE"), Expr::Bool(true));
        assert_eq!(parse("=1"), Expr::Number(1.0));
        assert_eq!(parse("='hi!'"), Expr::from("hi!"));
        assert_eq!(parse("=F.ADD(1, 2)"), Expr::Number(3.0));

        // outside of an Excel sheet
        for formula in [
            "=XFE1",
            "=A1048577",
            "=A1:A1048577",
            "=XFE:XFE",
            "=1048577:1",
            "=A99999999999",
        ] {
            let reference = formula.trim_start_matches('=');
            assert!(matches!(Formula::new(formula), Err(Error::Parser(r)) if r == reference));
        }
        for formula in ["=A0", "=AAAA1", "=A 1", "=A1 :B2", "=A1:B", "=Sheet 2!A1", "=A:1"] {
            assert!(Formula::new(formula).is_err(), "{formula}");
        }
    }
}
