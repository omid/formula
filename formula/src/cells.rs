use crate::Expr;

/// The cells that references in a formula read: rows of values, a workbook, a database…
///
/// ```rust
/// use formula::{Cells, Expr, ExprError, Formula, Size};
///
/// /// Every cell holds its row number
/// struct RowNumbers;
///
/// impl Cells for RowNumbers {
///     fn size(&self, sheet: Option<&str>) -> Option<Size> {
///         // only the formula's own sheet exists
///         sheet.is_none().then_some(Size { rows: 10, columns: 10 })
///     }
///
///     fn cell(&self, _sheet: Option<&str>, row: u32, _column: u32) -> Option<Expr> {
///         Some(Expr::from(f64::from(row)))
///     }
/// }
///
/// let formula = Formula::new("=F.ADD(A2, Sheet2!A3)").unwrap();
/// assert_eq!(formula.parse_with(&RowNumbers).unwrap(), Expr::Error(ExprError::Reference));
///
/// let formula = Formula::new("=SUM(A1:A3)").unwrap();
/// assert_eq!(formula.parse_with(&RowNumbers).unwrap(), Expr::Number(6.0));
/// ```
pub trait Cells {
    /// How many rows and columns of `sheet` have data, or `None` if there's no such sheet, which is `#REF!`.
    /// `sheet` is `None` for references without a sheet name, like `A1`.
    /// Whole columns and rows like `A:A` only read this area.
    fn size(&self, sheet: Option<&str>) -> Option<Size>;

    /// The value of a cell, `None` if it's empty. Only called for the sheets that `size` knows.
    /// Rows and columns start at 1, so `Sheet2!B3` is `cell(Some("Sheet2"), 3, 2)`.
    fn cell(&self, sheet: Option<&str>, row: u32, column: u32) -> Option<Expr>;
}

/// How many rows and columns of a sheet have data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Size {
    pub rows: u32,
    pub columns: u32,
}

/// One table as rows of values: `rows[0][0]` is `A1`, and an `Expr::Null` element is an empty cell.
/// References with a sheet name are `#REF!`.
impl Cells for [Vec<Expr>] {
    fn size(&self, sheet: Option<&str>) -> Option<Size> {
        let columns = self.iter().map(Vec::len).max().unwrap_or(0);
        sheet.is_none().then(|| Size {
            rows: u32::try_from(self.len()).unwrap_or(u32::MAX),
            columns: u32::try_from(columns).unwrap_or(u32::MAX),
        })
    }

    fn cell(&self, _sheet: Option<&str>, row: u32, column: u32) -> Option<Expr> {
        let row = self.get(usize::try_from(row).ok()?.checked_sub(1)?)?;
        match row.get(usize::try_from(column).ok()?.checked_sub(1)?)? {
            Expr::Null => None,
            value => Some(value.clone()),
        }
    }
}

impl Cells for Vec<Vec<Expr>> {
    fn size(&self, sheet: Option<&str>) -> Option<Size> {
        self.as_slice().size(sheet)
    }

    fn cell(&self, sheet: Option<&str>, row: u32, column: u32) -> Option<Expr> {
        self.as_slice().cell(sheet, row, column)
    }
}

impl<C: Cells + ?Sized> Cells for &C {
    fn size(&self, sheet: Option<&str>) -> Option<Size> {
        (**self).size(sheet)
    }

    fn cell(&self, sheet: Option<&str>, row: u32, column: u32) -> Option<Expr> {
        (**self).cell(sheet, row, column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rows_of_values() {
        let rows = vec![
            vec![Expr::from(1), Expr::Null],
            vec![Expr::from("a"), Expr::from(true), Expr::from(2)],
        ];

        assert_eq!(rows.size(None), Some(Size { rows: 2, columns: 3 }));
        assert_eq!(rows.size(Some("Sheet1")), None);
        assert_eq!(rows.cell(None, 1, 1), Some(Expr::Number(1.0)));
        assert_eq!(rows.cell(None, 2, 3), Some(Expr::Number(2.0)));
        // empty, outside of the data, and invalid positions
        assert_eq!(rows.cell(None, 1, 2), None);
        assert_eq!(rows.cell(None, 1, 3), None);
        assert_eq!(rows.cell(None, 3, 1), None);
        assert_eq!(rows.cell(None, 0, 1), None);
        assert_eq!(rows.cell(None, 1, 0), None);

        let no_rows: &[Vec<Expr>] = &[];
        assert_eq!(no_rows.size(None), Some(Size::default()));
    }
}
