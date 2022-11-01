use pdfium_render::prelude::*;
use crate::models::Part;

pub fn get_pdfium() -> Pdfium
{
    Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library()).unwrap())
}

pub fn add_page(new_document: &mut PdfDocument, source_document: PdfDocument, part: &Part) {
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
            new_document.pages()
                .copy_page_range_from_document(&source_document,
                    start_page_number - 1..=end_page_number - 1,
                    new_document.pages().len()
                ).unwrap()
        },
    }
}