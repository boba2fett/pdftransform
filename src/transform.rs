use pdfium_render::prelude::*;
use crate::models::{Part, Rotation};

pub fn get_pdfium() -> Pdfium
{
    Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library()).unwrap())
}

pub fn add_page(new_document: &mut PdfDocument, source_document: &mut PdfDocument, part: &Part) -> Result<(), &'static str> {
    let start_page_number = part.start_page_number.unwrap_or(1);
    let end_page_number = part.end_page_number.unwrap_or(source_document.pages().len());
    turn_single_page(start_page_number, end_page_number, source_document, part)?;
    new_document.pages()
    .copy_page_range_from_document(&source_document,
        start_page_number - 1..=end_page_number - 1,
        new_document.pages().len()
    ).map_err(|_| "Could not transfer pages.")?;
    Ok(())
}

fn turn_single_page(start_page_number: u16, end_page_number: u16, source_document: &mut PdfDocument, part: &Part) -> Result<(), &'static str> {
    if start_page_number == end_page_number && part.rotation.is_some() {
        let pages = source_document.pages();
        let mut page = pages.iter().nth((start_page_number - 1).into()).ok_or("Source document doesn't contain enoug pages.")?;
        let rotation = page.rotation().map_err(|_| "Could not get rotation.")?;
        let part_rotation: i32 = part.rotation.as_ref().unwrap_or(&Rotation::P0).as_degrees();
        let turn_rotation: i32 = {
            if part_rotation < 0 {
                360 + part_rotation
            }
            else {
                part_rotation
            }
        };
        let mut new_rotation = rotation.as_degrees() as i32 + turn_rotation;
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
    Ok(())
}