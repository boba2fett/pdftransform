use std::{sync::{Arc, Mutex, atomic::AtomicPtr}, ptr};

use pdfium_render::prelude::*;
use crate::models::Part;

pub static mut PDFIUM_PTR: AtomicPtr<Arc<Pdfium>> = AtomicPtr::new(ptr::null_mut());

pub fn get_pdfium() -> &'static Arc<Pdfium>
{
    unsafe {
        let a: &mut *mut Arc<Pdfium> = PDFIUM_PTR.get_mut();
        &**a
    }
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
        (Some(_), None) => todo!(),
        (Some(_), Some(_)) => todo!(),
    }
}