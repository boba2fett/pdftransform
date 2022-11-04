use pdfium_render::prelude::*;
use crate::models::{Part, Rotation};

pub fn get_pdfium() -> Pdfium
{
    Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library()).unwrap())
}

pub fn add_page(new_document: &mut PdfDocument, source_document: &mut PdfDocument, part: &Part) {
    match (part.start_page_number, part.end_page_number) {
        (None, None) => {
            new_document.pages().append(&source_document).unwrap()
        },
        (None, Some(end_page_number)) => {
            new_document.pages()
                .copy_page_range_from_document(&source_document,
                    0..=end_page_number - 1,
                    new_document.pages().len()
                ).unwrap()
        },
        (Some(start_page_number), None) => {
            new_document.pages()
                .copy_page_range_from_document(&source_document,
                    start_page_number - 1..=source_document.pages().len() - 1,
                    new_document.pages().len()
                ).unwrap()
        },
        (Some(start_page_number), Some(end_page_number)) => {
            if start_page_number == end_page_number && part.rotation.is_some() {
                {
                    let pages = source_document.pages();
                    let mut page = pages.iter().nth((start_page_number - 1).into()).unwrap();
                    let rotation = page.rotation().unwrap();
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
                new_document.pages().copy_page_from_document(&source_document, start_page_number - 1, new_document.pages().len());
            }
            new_document.pages()
                .copy_page_range_from_document(&source_document,
                    start_page_number - 1..=end_page_number - 1,
                    new_document.pages().len()
                ).unwrap()
        },
    }
}