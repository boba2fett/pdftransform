use pdfium_render::prelude::*;
use crate::models::{Part, Rotation};

pub fn init_pdfium() -> Pdfium {
    Pdfium::new(Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")).unwrap())
}

pub fn add_part(new_document: &mut PdfDocument, source_document: &PdfDocument, part: &Part) -> Result<bool, &'static str> {
    let start_page_number = part.start_page_number.unwrap_or(1);
    let end_page_number = part.end_page_number.unwrap_or(source_document.pages().len());
    validate_pages(start_page_number, end_page_number, source_document)?;
    
    let turned = turn_single_pages(start_page_number, end_page_number, source_document, part)?;
    new_document.pages()
    .copy_page_range_from_document(&source_document,
        start_page_number - 1..=end_page_number - 1,
        new_document.pages().len()
    ).map_err(|_| "Could not transfer pages.")?;
    
    Ok(turned)
}

fn validate_pages(start_page_number: u16, end_page_number: u16, source_document: &PdfDocument) -> Result<(), &'static str> {
    if start_page_number > end_page_number {
        return Err("Start page number can't be greater than end page number.")
    }
    let pages = source_document.pages();
    if end_page_number > pages.len() {
        return Err("End page number exceeds pages of document.")
    }
    Ok(())
}

fn turn_single_pages(start_page_number: u16, end_page_number: u16, source_document: &PdfDocument, part: &Part) -> Result<bool, &'static str> {
    if part.rotation.is_some() {
        let part_rotation: i32 = part.rotation.unwrap_or(Rotation::P0).as_degrees();
        let part_rotation: i32 = {
            if part_rotation < 0 {
                360 + part_rotation
            }
            else {
                part_rotation
            }
        };
        let pages = source_document.pages();
        for mut page in pages.iter().skip(start_page_number as usize - 1).take(end_page_number as usize - start_page_number as usize + 1) {
            let rotation = page.rotation().map_err(|_| "Could not get rotation.")?;
            let mut new_rotation = rotation.as_degrees() as i32 + part_rotation;
            if new_rotation > 360 {
                new_rotation -= 360;
            }
            let new_rotation = match new_rotation {
                0 => PdfBitmapRotation::None,
                90 => PdfBitmapRotation::Degrees90,
                180 => PdfBitmapRotation::Degrees180,
                270 => PdfBitmapRotation::Degrees270,
                _ => PdfBitmapRotation::None,
            };
            page.set_rotation(new_rotation);
        }
        Ok(true)
    }
    else {
        Ok(false)
    }
}
