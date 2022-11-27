use std::path::PathBuf;
use image::ImageFormat;
use mongodb::Client;
use pdfium_render::{render_config::PdfRenderConfig, prelude::PdfDocument};
use tokio::fs;

use crate::{models::{PreviewResult, PreviewPageResult, PreviewAttachmentResult, PreviewSignature}, transform::init_pdfium, files::{TempJobFileProvider, store_result_file}, routes::file_route};

pub async fn get_preview(client: &Client, job_id: &str, token: &str, source_file: &PathBuf, file_provider: &TempJobFileProvider) -> Result<PreviewResult, &'static str> {
    let results: (Vec<_>, Vec<_>, Vec<_>, bool) = {
        let pdfium = init_pdfium();
        let document = pdfium.load_pdf_from_file(&source_file, None).map_err(|_| "Could not open document.")?;

        let render_config = PdfRenderConfig::new();
        let pages = document.pages().iter().enumerate().map(|(index, page)| -> Result<_, &'static str> {
            let path = file_provider.get_path();
            page.render_with_config(&render_config).map_err(|_| "Could not render to image.")?
            .as_image().as_rgba8().ok_or("Could not render image.")?
            .save_with_format(&path, ImageFormat::Jpeg).map_err(|_| "Could not save image.")?;
            let page_number = format!("{}", index + 1);
            
            Ok(async move {
                let file = fs::read(path).await.map_err(|_| "Could not read file.")?;
                let file_id = store_result_file(&client, &job_id, &token,&page_number, Some("image/jpeg"),&*file).await?;
                Ok::<PreviewPageResult, &'static str>(PreviewPageResult {
                    download_url: file_route(&file_id, &token)
                })
            })
        }).collect();
        let attachments: Vec<_> = document.attachments().iter().map(|attachment| -> Result<_, &'static str> {
            let name = attachment.name();
            let bytes = attachment.save_to_bytes().map_err(|_| "Could not save attachment.")?;
            
            Ok(async move {
                let file_id = store_result_file(&client, &job_id, &token, &name, None, &*bytes).await?;
                Ok::<PreviewAttachmentResult, &'static str>(PreviewAttachmentResult {
                    name,
                    download_url: file_route(&file_id, &token)
                })
            })
        }).collect();
        (pages, attachments, signatures(&document), is_protected(&document).unwrap_or(false))
    };
    let mut preview_page_results = Vec::with_capacity(results.0.len());
    for result in results.0 {
        let value = result?.await?;
        preview_page_results.push(value);
    }
    let mut preview_attachment_results = Vec::with_capacity(results.1.len());
    for result in results.1 {
        let value = result?.await?;
        preview_attachment_results.push(value);
    }
    Ok(PreviewResult {
        page_count: preview_page_results.len(),
        pages: preview_page_results,
        attachments: preview_attachment_results,
        signatures: results.2,
        protected: results.3,
    })
}

fn is_protected(document: &PdfDocument) -> Result<bool, &'static str> {
    let permissions = document.permissions();
    let protected = !permissions.can_add_or_modify_text_annotations().map_err(|_| "Could not determine permissions.")?
        || !permissions.can_assemble_document().map_err(|_| "Could not determine permissions.")?
        || !permissions.can_create_new_interactive_form_fields().map_err(|_| "Could not determine permissions.")?
        || !permissions.can_extract_text_and_graphics().map_err(|_| "Could not determine permissions.")?
        || !permissions.can_fill_existing_interactive_form_fields().map_err(|_| "Could not determine permissions.")?
        || !permissions.can_modify_document_content().map_err(|_| "Could not determine permissions.")?;
    Ok(protected)
}

fn signatures(document: &PdfDocument) -> Vec<PreviewSignature> {
    document.signatures().iter().map(|signature| PreviewSignature {
        signing_date: signature.signing_date(),
        reason: signature.reason(),
        signature: signature.bytes(),
    }).collect()
}
