use hashbrown::HashMap;
use rayon::prelude::*;
use rust_xlsxwriter::{DocProperties, Format, FormatAlign, Workbook, Worksheet, XlsxSerialize};
use serde::Serialize;
use std::{path::PathBuf, sync::LazyLock};

use crate::MyResult;

// Install fonts:
// "DejaVu Sans Mono": pacman -S ttf-dejavu
// "Noto Sans Mono"  : pacman -S ttf-liberation
// "Liberation Mono" : pacman -S noto-fonts

const HEADER_SIZE: f64 = 11.0;
const FONT_SIZE: f64 = 12.0;
const FONT_NAME: &str = "Liberation Mono";
const MAX_NUMBER_OF_ROWS: usize = 1_000_000;

// See the application in structures::path_info::PathInfo

/// Write XLSX File according to some struct T
///
/// The lines (or rows) are given by `&[T]`
///
/// <https://docs.rs/rust_xlsxwriter/latest/rust_xlsxwriter/serializer/index.html>
pub fn write_xlsx<T>(lines: &[T], sheet_name: &str, path: PathBuf) -> MyResult<()>
where
    T: Serialize + XlsxSerialize + Sync,
{
    if lines.is_empty() {
        return Ok(());
    }

    // Each chunk divides the slice &[T] into smaller slices.
    let worksheets: MyResult<Vec<Worksheet>> = lines
        .par_chunks(MAX_NUMBER_OF_ROWS) // rayon parallel iterator
        .enumerate()
        .map(|(index, data)| -> MyResult<Worksheet> {
            //println!("thread id: {:?}", std::thread::current().id());
            let mut new_name = sheet_name.to_string();

            if index >= 1 {
                new_name = format!("{} {}", sheet_name, index + 1);
            }

            // Get worksheet with sheet name.
            let worksheet: Worksheet = get_worksheet(data, &new_name)?;

            Ok(worksheet)
        })
        .collect();

    // Create a new Excel file object.
    let mut workbook = Workbook::new();
    let properties = get_properties()?;
    workbook.set_properties(&properties);

    worksheets?.into_iter().for_each(|worksheet| {
        workbook.push_worksheet(worksheet);
    });

    // Save the workbook to disk.
    workbook.save(&path).inspect_err(|xlsx_error| {
        // Add a custom error message
        eprintln!("fn write_xlsx()");
        eprintln!("Failed to write XLSX file {path:?}");
        eprintln!("Error: {xlsx_error}");
    })?;

    Ok(())
}

fn get_properties() -> MyResult<DocProperties> {
    // Add it to the document metadata.
    let properties = DocProperties::new()
        .set_title("Find Identical Files")
        .set_subject("Find identical files according to their size and hashing algorithm")
        .set_author("Claudio FSR (https://github.com/claudiofsr/find-identical-files)")
        .set_keywords("find, identical, hash algorithm")
        .set_comment("Built with Rust")
        .set_hyperlink_base("https://github.com/claudiofsr/find-identical-files");

    Ok(properties)
}

/// Get Worksheet according to some struct T
fn get_worksheet<T>(lines: &[T], sheet_name: &str) -> MyResult<Worksheet>
where
    T: Serialize + XlsxSerialize,
{
    let mut worksheet = Worksheet::new();
    let fmt_header = get_xlsx_format("header");

    worksheet
        .set_name(sheet_name)?
        .set_row_height(0, 32)?
        .set_row_format(0, fmt_header)?
        .set_freeze_panes(1, 0)?;

    // Set up the start location and headers of the data to be serialized.
    worksheet.set_serialize_headers::<T>(0, 0)?;

    // Serialize the data.
    worksheet.serialize(&lines)?;

    worksheet.autofit();

    Ok(worksheet)
}

/// XLSX Formats
static XLSX_FORMATS: LazyLock<HashMap<&'static str, Format>> = LazyLock::new(|| {
    let fmt_header: Format = Format::new()
        .set_align(FormatAlign::Center) // horizontally
        .set_align(FormatAlign::VerticalCenter)
        .set_text_wrap()
        .set_font_size(HEADER_SIZE);

    let fmt_center = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_font_name(FONT_NAME)
        .set_font_size(FONT_SIZE);

    let fmt_integer = Format::new()
        .set_align(FormatAlign::VerticalCenter)
        .set_num_format("#,##0")
        .set_font_name(FONT_NAME)
        .set_font_size(FONT_SIZE);

    let fmt_default = Format::new()
        .set_align(FormatAlign::VerticalCenter)
        .set_font_name(FONT_NAME)
        .set_font_size(FONT_SIZE);

    let formats = [
        ("header", fmt_header),
        ("center", fmt_center),
        ("integer", fmt_integer),
        ("default", fmt_default),
    ];

    HashMap::from(formats)
});

/// Get XLSX format
///
/// Add some formats to use with the serialization data.
pub fn get_xlsx_format(name: &str) -> &Format {
    match XLSX_FORMATS.get(name) {
        Some(format) => format,
        None => panic!("Format {name} not defined!"),
    }
}
