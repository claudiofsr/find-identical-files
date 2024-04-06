use hashbrown::HashMap;
use rust_xlsxwriter::{Format, FormatAlign, Table, Workbook, Worksheet};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::MyResult;

const HEADER_SIZE: f64 = 13.0;
const FONT_SIZE: f64 = 12.0;
const MAX_NUMBER_OF_ROWS: usize = 1_000_000;

/// Write XLSX File according to some struct T
///
/// The lines (or rows) are given by &[T]
///
/// <https://docs.rs/rust_xlsxwriter/latest/rust_xlsxwriter/serializer/index.html>
pub fn write_xlsx<'de, T>(lines: &[T], sheet_name: &str, path: PathBuf) -> MyResult<()>
where
    T: Serialize + Deserialize<'de>,
{
    if lines.is_empty() {
        return Ok(());
    }

    // Create a new Excel file object.
    let mut workbook = Workbook::new();

    // Split a vector into smaller vectors of size N
    for (index, data) in lines.chunks(MAX_NUMBER_OF_ROWS).enumerate() {
        let mut new_name = sheet_name.to_string();

        if index >= 1 {
            new_name = format!("{} {}", sheet_name, index + 1);
        }

        // Get worksheet with sheet name.
        let worksheet: Worksheet = get_worksheet(data, &new_name)?;

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
fn get_worksheet<'de, T>(lines: &[T], sheet_name: &str) -> MyResult<Worksheet>
where
    T: Serialize + Deserialize<'de>,
{
    let mut worksheet = Worksheet::new();

    // Add some formats to use with the serialization data.
    let fmt: HashMap<&str, Format> = create_formats();

    worksheet
        .set_name(sheet_name)?
        .set_row_height(0, 32)?
        .set_row_format(0, fmt.get("header").unwrap())?
        .set_freeze_panes(1, 0)?;

    // Set up the start location and headers of the data to be serialized.
    worksheet.deserialize_headers::<T>(0, 0)?;

    // Create and configure a new table.
    // Why LibreOffice Calc not recognize the table styles?
    let table = Table::new().set_autofilter(true).set_total_row(false);

    let row_number: u32 = lines.len().try_into()?;
    let column_number: u16 = 5; // See PathInfo

    for k in 0..column_number {
        if [0, 3, 4].contains(&k) {
            worksheet.set_column_format(k, fmt.get("integer").unwrap())?;
        } else if k == 1 {
            worksheet.set_column_format(k, fmt.get("center").unwrap())?;
        } else {
            worksheet.set_column_format(k, fmt.get("default").unwrap())?;
        }
    }

    // Add the table to the worksheet.
    worksheet.add_table(0, 0, row_number, column_number - 1, &table)?;

    for line in lines {
        // Serialize the data.
        worksheet.serialize(line)?;
    }

    worksheet.autofit();

    Ok(worksheet)
}

/// Add some formats to use with the serialization data.
fn create_formats() -> HashMap<&'static str, Format> {
    let fmt_header: Format = Format::new()
        .set_align(FormatAlign::Center) // horizontally
        .set_align(FormatAlign::VerticalCenter)
        .set_text_wrap()
        .set_font_size(HEADER_SIZE);

    let fmt_center = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_font_name("DejaVu Sans Mono") // "Hack"
        .set_font_size(FONT_SIZE);

    let fmt_integer = Format::new()
        .set_align(FormatAlign::VerticalCenter)
        .set_num_format("#,##0")
        .set_font_name("DejaVu Sans Mono") // "Hack"
        .set_font_size(FONT_SIZE);

    let fmt_default = Format::new()
        .set_align(FormatAlign::VerticalCenter)
        .set_font_name("DejaVu Sans Mono") // "Hack"
        .set_font_size(FONT_SIZE);

    HashMap::from([
        ("header", fmt_header),
        ("center", fmt_center),
        ("integer", fmt_integer),
        ("default", fmt_default),
    ])
}
