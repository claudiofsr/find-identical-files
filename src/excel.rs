use hashbrown::HashMap;
use rayon::prelude::*;
use rust_xlsxwriter::{DocProperties, Format, FormatAlign, Workbook, Worksheet, XlsxSerialize};
use serde::Serialize;
use std::{path::PathBuf, sync::LazyLock};

use crate::{FIFError, FIFResult};

// Install fonts:
// "DejaVu Sans Mono": pacman -S ttf-dejavu
// "Noto Sans Mono"  : pacman -S ttf-liberation
// "Liberation Mono" : pacman -S noto-fonts

const HEADER_SIZE: f64 = 11.0;
const FONT_SIZE: f64 = 12.0;
const FONT_NAME: &str = "Liberation Mono";

/// Excel has a limit of 1,048,576 rows. We use 1M to stay safe.
const MAX_NUMBER_OF_ROWS: usize = 1_000_000;

// See the application in structures::path_info::PathInfo

/// Writes a collection of serializable data to an XLSX file.
///
/// If the data exceeds `MAX_NUMBER_OF_ROWS`, it automatically splits
/// the content into multiple worksheets.
pub fn write_xlsx<T>(lines: &[T], sheet_name: &str, path: PathBuf) -> FIFResult<()>
where
    T: Serialize + XlsxSerialize + Sync,
{
    if lines.is_empty() {
        return Ok(());
    }

    // Each chunk divides the slice &[T] into smaller slices.
    let worksheets: FIFResult<Vec<Worksheet>> = lines
        .par_chunks(MAX_NUMBER_OF_ROWS) // rayon parallel iterator
        .enumerate()
        .map(|(index, data)| -> FIFResult<Worksheet> {
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

    // Add all generated worksheets to the workbook.
    for worksheet in worksheets? {
        workbook.push_worksheet(worksheet);
    }

    // Save the workbook to disk.
    workbook.save(&path).inspect_err(|xlsx_error| {
        // Add a custom error message
        eprintln!("fn write_xlsx()");
        eprintln!("Failed to write XLSX file {path:?}");
        eprintln!("Error: {xlsx_error}");
    })?;

    Ok(())
}

fn get_properties() -> FIFResult<DocProperties> {
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
fn get_worksheet<T>(lines: &[T], sheet_name: &str) -> FIFResult<Worksheet>
where
    T: Serialize + XlsxSerialize,
{
    let mut worksheet = Worksheet::new();
    let fmt_header = get_xlsx_format("header")?;

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

/// Pre-defined Excel formats stored in a thread-safe LazyLock.
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

/// Safely retrieves a format by its name.
///
/// # Errors
/// Returns `FIFError::InvalidXlsxFormat` if the format name is not recognized.
pub fn get_xlsx_format(name: &str) -> FIFResult<&Format> {
    XLSX_FORMATS
        .get(name)
        .ok_or_else(|| FIFError::InvalidXlsxFormat(name.to_string()))
}

/// Bridge function for XlsxSerialize macro to get the integer format.
pub fn fmt_integer() -> Format {
    get_xlsx_format("integer").cloned().unwrap_or_default()
}

/// Bridge function for XlsxSerialize macro to get the center format.
pub fn fmt_center() -> Format {
    get_xlsx_format("center").cloned().unwrap_or_default()
}

/// Bridge function for XlsxSerialize macro to get the default format.
pub fn fmt_default() -> Format {
    get_xlsx_format("default").cloned().unwrap_or_default()
}

#[allow(dead_code)]
/// Bridge function for XlsxSerialize macro to get the header format.
pub fn fmt_header() -> Format {
    get_xlsx_format("header").cloned().unwrap_or_default()
}
