use std::path::PathBuf;

use bson::DateTime;
use image::ImageFormat;
use mongodb::Client;
use pdfium_render::{render_config::PdfRenderConfig, prelude::PdfDocument};
use rocket::form::validate::len;
use tokio::fs;

use crate::{models::{PreviewResult, PreviewModel, PreviewPageResult}, transform::init_pdfium, persistence::{generate_30_alphanumeric, save_new_preview}, files::{TempJobFileProvider, store_result_file}, routes::preview_file_route};

pub async fn get_preview(client: &Client, file: PathBuf) -> Result<PreviewResult, &'static str> {
    let token = generate_30_alphanumeric();
    let saved_job = save_new_preview(&client, PreviewModel {
        id: None,
        token: token.clone(),
        created: DateTime::now()
    }).await?;
    let id = saved_job.id.unwrap().to_string();
    let id = &id;
    let token = &token;

    let file_provider = TempJobFileProvider::build(&id).await;
    let results: Vec<_> = {
        let pdfium = init_pdfium();
        let document = pdfium.load_pdf_from_file(&file, None).map_err(|_| "Could not open document.")?;

        let render_config = PdfRenderConfig::new();
        let pages = document.pages();
        let len = pages.len();
        pages.iter().enumerate().map(|(index, page)| -> Result<_, &'static str> {
            let path = file_provider.get_path();
            page.render_with_config(&render_config).map_err(|_| "Could not render to image.")?
            .as_image().as_rgba8().ok_or("Could not render image.")?
            .save_with_format(&path, ImageFormat::Jpeg).map_err(|_| "Could not save image.")?;
            let page_number = format!("{}", index + 1);

            Ok(async move {
                let file = fs::read(path).await.map_err(|_| "Could not read file.")?;
                let file_id = store_result_file(&client, &page_number, &*file).await?;
                Ok::<PreviewPageResult, &'static str>(PreviewPageResult {
                    download_url: preview_file_route(&id, &file_id, &token)
                })
            })
        }).collect()
    };
    let mut preview_results = Vec::with_capacity(results.len());
    for result in results {
        let value = result?.await?;
        preview_results.push(value);
    }
    file_provider.clean_up().await;
    _ = fs::remove_file(&file).await;
    Ok(PreviewResult {
        pages: preview_results
    })
}

// fn is_protected(document: &PdfDocument) -> Result<bool, &'static str> {
//     let permissions = document.permissions();
//     let protected = !permissions.can_add_or_modify_text_annotations().map_err(|_| "Could not determine permissions.")?
//         || !permissions.can_assemble_document().map_err(|_| "Could not determine permissions.")?
//         || !permissions.can_create_new_interactive_form_fields().map_err(|_| "Could not determine permissions.")?
//         || !permissions.can_extract_text_and_graphics().map_err(|_| "Could not determine permissions.")?
//         || !permissions.can_fill_existing_interactive_form_fields().map_err(|_| "Could not determine permissions.")?
//         || !permissions.can_modify_document_content().map_err(|_| "Could not determine permissions.")?;
//     Ok(protected)
// }

// fn signatures(document: &PdfDocument) {
//     let signatures: Vec<(Vec<u8>)> = document.signatures().iter().map(|signature| signature.bytes()).collect();
// }
