use image::ImageFormat;
use pdfium_render::{
    prelude::{PdfDocument, Pdfium},
    render_config::PdfRenderConfig,
};
use std::{io::Cursor, sync::Arc};

use common::{
    models::{PreviewAttachmentResult, PreviewPageResult, PreviewResult, PreviewSignature}, persistence::IFileStorage,
};

#[cfg(feature = "static")]
pub fn init_pdfium() -> Result<Pdfium, &'static str> {
    Ok(Pdfium::new(Pdfium::bind_to_statically_linked_library().map_err(|_| "Could not init pdfium")?))
}

#[cfg(not(feature = "static"))]
pub fn init_pdfium() -> Result<Pdfium, &'static str> {
    Ok(Pdfium::new(Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")).map_err(|_| "Could not init pdfium")?))
}

#[async_trait::async_trait]
pub trait IPreviewService: Send + Sync {
    async fn get_preview(&self, job_id: &str, token: &str, source_file: Vec<u8>) -> Result<PreviewResult, &'static str>;
}

pub struct PreviewService {
    pub storage: Arc<dyn IFileStorage>,
    pub pdfium: Pdfium,
}

#[async_trait::async_trait]
impl IPreviewService for PreviewService {
    async fn get_preview(&self, job_id: &str, token: &str, source_file: Vec<u8>) -> Result<PreviewResult, &'static str> {
        let results: (Vec<_>, Vec<_>, Vec<_>, bool) = {
            let document = self.pdfium.load_pdf_from_byte_vec(source_file, None).map_err(|_| "Could not open document.")?;

            let render_config = PdfRenderConfig::new();
            let pages = document
                .pages()
                .iter()
                .enumerate()
                .map(|(index, page)| -> Result<_, &'static str> {
                    let mut bytes: Vec<u8> = Vec::new();
                    page.render_with_config(&render_config)
                        .map_err(|_| "Could not render to image.")?
                        .as_image()
                        .as_rgba8()
                        .ok_or("Could not render image.")?
                        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
                        .map_err(|_| "Could not save image.")?;
                    let page_number = format!("{}", index + 1);
                    let text = page.text().map_err(|_| "")?.all();

                    Ok(async move {
                        let file_url = self.storage.store_result_file(&format!("{}-{}", &job_id, &page_number), &format!("{}.png", page_number), Some("image/png"), bytes).await?;
                        Ok::<PreviewPageResult, &'static str>(PreviewPageResult {
                            download_url: file_url,
                            text,
                        })
                    })
                })
                .collect();
            let attachments: Vec<_> = document
                .attachments()
                .iter()
                .map(|attachment| -> Result<_, &'static str> {
                    let name = attachment.name();
                    let bytes = attachment.save_to_bytes().map_err(|_| "Could not save attachment.")?;

                    Ok(async move {
                        let file_url = self.storage.store_result_file(&format!("{}-{}", &job_id, &name), &name, None, bytes).await?;
                        Ok::<PreviewAttachmentResult, &'static str>(PreviewAttachmentResult {
                            name,
                            download_url: file_url,
                        })
                    })
                })
                .collect();
            (pages, attachments, self.signatures(&document), self.is_protected(&document).unwrap_or(false))
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
}

impl PreviewService {
    fn is_protected(&self, document: &PdfDocument) -> Result<bool, &'static str> {
        let permissions = document.permissions();
        let protected = !permissions.can_add_or_modify_text_annotations().map_err(|_| "Could not determine permissions.")?
            || !permissions.can_assemble_document().map_err(|_| "Could not determine permissions.")?
            || !permissions.can_create_new_interactive_form_fields().map_err(|_| "Could not determine permissions.")?
            || !permissions.can_extract_text_and_graphics().map_err(|_| "Could not determine permissions.")?
            || !permissions.can_fill_existing_interactive_form_fields().map_err(|_| "Could not determine permissions.")?
            || !permissions.can_modify_document_content().map_err(|_| "Could not determine permissions.")?;
        Ok(protected)
    }

    fn signatures(&self, document: &PdfDocument) -> Vec<PreviewSignature> {
        document
            .signatures()
            .iter()
            .map(|signature| PreviewSignature {
                signing_date: signature.signing_date(),
                reason: signature.reason(),
                signature: signature.bytes(),
            })
            .collect()
    }
}
