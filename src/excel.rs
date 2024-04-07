use rust_xlsxwriter::{Format, FormatAlign, Workbook, Worksheet, XlsxSerialize};
use serde::Serialize;
use std::path::PathBuf;

use crate::MyResult;

pub const HEADER_SIZE: f64 = 11.0;
pub const FONT_SIZE: f64 = 12.0;
pub const FONT_NAME: &str = "DejaVu Sans Mono"; // "Hack"
const MAX_NUMBER_OF_ROWS: usize = 1_000_000;

/// Write XLSX File according to some struct T
///
/// The lines (or rows) are given by &[T]
///
/// <https://docs.rs/rust_xlsxwriter/latest/rust_xlsxwriter/serializer/index.html>
pub fn write_xlsx<T>(lines: &[T], sheet_name: &str, path: PathBuf) -> MyResult<()>
where
    T: Serialize + XlsxSerialize,
{
    if lines.is_empty() {
        return Ok(());
    }

    // Create a new Excel file object.
    let mut workbook = Workbook::new();

    let fmt_header: Format = Format::new()
        .set_align(FormatAlign::Center) // horizontally
        .set_align(FormatAlign::VerticalCenter)
        .set_text_wrap()
        .set_font_size(HEADER_SIZE);

    // Split a vector into smaller vectors of size N
    for (index, data) in lines.chunks(MAX_NUMBER_OF_ROWS).enumerate() {
        let mut new_name = sheet_name.to_string();

        if index >= 1 {
            new_name = format!("{} {}", sheet_name, index + 1);
        }

        // Get worksheet with sheet name.
        let worksheet: Worksheet = get_worksheet(data, &fmt_header, &new_name)?;

        workbook.push_worksheet(worksheet);
    }

    // Save the workbook to disk.
    workbook.save(&path).map_err(|xlsx_error| {
        // Add a custom error message
        eprintln!("fn write_xlsx()");
        eprintln!("Error: Failed to write XLSX file {:?}", path);
        xlsx_error
    })?;

    Ok(())
}

/// Get Worksheet according to some struct T
fn get_worksheet<T>(lines: &[T], fmt_header: &Format, sheet_name: &str) -> MyResult<Worksheet>
where
    T: Serialize + XlsxSerialize,
{
    let mut worksheet = Worksheet::new();

    worksheet
        .set_name(sheet_name)?
        .set_row_height(0, 32)?
        .set_row_format(0, fmt_header)?
        .set_freeze_panes(1, 0)?;

    // Set up the start location and headers of the data to be serialized.
    worksheet.set_serialize_headers::<T>(0, 0)?;

    for line in lines {
        // Serialize the data.
        worksheet.serialize(line)?;
    }

    worksheet.autofit();

    Ok(worksheet)
}
