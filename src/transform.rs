use pdfium_render::prelude::*;
use crate::{models::{Part, Rotation, TransformDocumentResult, Document}, files::store_result_file, routes::file_route, download::DownloadedSourceFile};

pub fn init_pdfium() -> Pdfium {
    Pdfium::new(Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")).unwrap())
}

pub async fn get_transformation<'a>(db_client: &mongodb::Client, job_id: &str, token: &str, documents: &Vec<Document>, source_files: Vec<&DownloadedSourceFile>) -> Result<Vec<TransformDocumentResult>, &'static str> {
    let results: Vec<_> = {
        let pdfium = init_pdfium();
        let mut cache: Option<(&str, PdfDocument)> = None;
            
        documents.iter().map(|document| -> Result<_, &'static str> {
            let cache_ref: &mut Option<(&str, PdfDocument)> = &mut cache;
            let bytes = {
                let mut new_doc = pdfium.create_new_pdf().map_err(|_| "Could not create document.")?;
                for part in &document.parts {
                    if cache_ref.is_some() && cache_ref.as_ref().unwrap().0.eq(&part.source_file) {
                        add_part(&mut new_doc, &cache_ref.as_ref().unwrap().1, part)?;
                    }
                    else {
                        let source_file = source_files.iter().find(|source_file| source_file.id.eq(&part.source_file)).ok_or("Could not find corresponding source file.")?;
                        let source_doc = pdfium.load_pdf_from_file(&source_file.path, None).map_err(|_| "Could not create document.")?;
                        *cache_ref = Some((&part.source_file, source_doc));
                        add_part(&mut new_doc, &cache_ref.as_ref().unwrap().1, part)?;
                    }
                }
                for attachment in &document.attachments {
                    let source_file = source_files.iter().find(|source_file| source_file.id.eq(&attachment.source_file)).ok_or("Could not find corresponding source file.")?;
                    new_doc.attachments_mut().create_attachment_from_file(&attachment.name, &source_file.path).map_err(|_| "Could not add attachment.")?;
                }
                new_doc.save_to_bytes().map_err(|_| "Could not save file.")?
            };
            Ok(async move {
                let file_id = store_result_file(db_client, &job_id, &token, &document.id, Some("application/pdf"),&*bytes).await?;

                Ok::<TransformDocumentResult, &'static str>(TransformDocumentResult {
                    download_url: file_route(&file_id, token),
                    id: document.id.to_string(),
                })
            })
        }).collect()
    };
    let mut document_results = Vec::with_capacity(documents.len());
    for result in results {
        let value = result?.await?;
        document_results.push(value);
    }
    Ok(document_results)
}

pub fn add_part(new_document: &mut PdfDocument, source_document: &PdfDocument, part: &Part) -> Result<(), &'static str> {
    let start_page_number = part.start_page_number.unwrap_or(1);
    let end_page_number = part.end_page_number.unwrap_or(source_document.pages().len());
    validate_pages(start_page_number, end_page_number, source_document)?;

    let new_start_page_number = new_document.pages().len() + 1;
    let new_end_page_number = new_start_page_number + (end_page_number - start_page_number);
    
    new_document.pages()
    .copy_page_range_from_document(&source_document,
        start_page_number - 1..=end_page_number - 1,
        new_start_page_number - 1
    ).map_err(|_| "Could not transfer pages.")?;
    
    turn_pages(new_start_page_number, new_end_page_number, new_document, part)?;

    Ok(())
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

fn turn_pages(start_page_number: u16, end_page_number: u16, source_document: &PdfDocument, part: &Part) -> Result<(), &'static str> {
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
    }
    Ok(())
}
