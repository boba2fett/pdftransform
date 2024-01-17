use std::{sync::Arc, io::Cursor};

use image::ImageFormat;
use pdfium_render::{
    prelude::{PdfDocument, Pdfium},
    render_config::PdfRenderConfig,
};

use common::{
    models::{PreviewAttachmentResult, PreviewPageResult, PreviewResult, PreviewSignature, PreviewJobModel}, persistence::IFileStorage,
};

pub fn init_pdfium() -> Result<Pdfium, &'static str> {
    Ok(Pdfium::new(Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")).map_err(|_| "Could not init pdfium")?))
}

#[async_trait::async_trait]
pub trait IPreviewService: Send + Sync {
    async fn get_preview(&self, job: &PreviewJobModel, source_file: Vec<u8>) -> Result<PreviewResult, &'static str>;
}

pub struct PreviewService {
    pub storage: Arc<dyn IFileStorage>,
    pub pdfium: Pdfium,
}

#[async_trait::async_trait]
impl IPreviewService for PreviewService {
    async fn get_preview(&self, job: &PreviewJobModel, source_file: Vec<u8>) -> Result<PreviewResult, &'static str> {
        let results: (usize, Option<_>, Option<Vec<_>>, Option<Vec<_>>, Option<Vec<_>>, bool) = {
            let job_id = &job.id;

            let document = self.pdfium.load_pdf_from_byte_vec(source_file, None).map_err(|_| "Could not open document.")?;
            let page_count = document.pages().len() as usize;
            let pages = match job.input.png || job.input.text {
                true => {
                    let start_page_number = job.input.start_page_number.unwrap_or(1);
                    let end_page_number = job.input.end_page_number.unwrap_or(document.pages().len());
                    self.validate_pages(start_page_number, end_page_number, &document)?;

                    let render_config = PdfRenderConfig::new();
                    Some(document
                        .pages()
                        .iter()
                        .skip((start_page_number - 1) as usize)
                        .take((end_page_number - start_page_number + 1) as usize)
                        .enumerate()
                        .map(|(index, page)| -> Result<_, &'static str> {
                            let image = match job.input.png {
                                true => {
                                    let mut bytes: Vec<u8> = Vec::new();
                                    page.render_with_config(&render_config)
                                        .map_err(|_| "Could not render to image.")?
                                        .as_image()
                                        .as_rgba8()
                                        .ok_or("Could not render image.")?
                                        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
                                        .map_err(|_| "Could not save image.")?;
                                    Some(bytes)
                                },
                                false => None,
                            };

                            let page_number = format!("{}", index + 1);

                            let text = match job.input.text {
                                true => Some(page.text().map_err(|_|"")?.all()),
                                false => None,
                            };

                            Ok(async move {
                                let file_url = match job.input.png {
                                    true => Some(self.storage.store_result_file(&format!("{}-{}", &job_id, &page_number), &format!("{}.png", page_number), Some("image/png"), image.unwrap()).await?),
                                    false => None,
                                };
                                Ok::<PreviewPageResult, &'static str>(PreviewPageResult {
                                    download_url: file_url,
                                    text,
                                })
                            })
                        })
                        .collect())
                    },
                false => None,
            };

            let attachments = match job.input.png {
                true => {
                    Some(document
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
                        .collect())
                    },
                false => None,
            };

            let signatures = match job.input.signatures {
                true => Some(self.signatures(&document)),
                false => None,
            };

            let protected = self.is_protected(&document).unwrap_or(false);

            let download_url = match job.input.pdf {
                true => Some(async move {
                    let file_url = self.storage.store_result_file(&job_id, "input.pdf", Some("application/pdf"), document.save_to_bytes().map_err(|_| "could not save")?).await?;
                    Ok::<_, &'static str>(file_url)
                }),
                false => None,
            };
            
            (page_count, download_url, pages, attachments, signatures, protected)
        };

        let pages = match results.2 {
            None => None,
            Some(pages) => {
                let mut ready_pages = Vec::with_capacity(pages.len());
                for result in pages {
                    let value = result?.await?;
                    ready_pages.push(value);
                }
                Some(ready_pages)
            }
        };

        let attachments = match results.3 {
            None => None,
            Some(attachments) => {
                let mut ready_attachments = Vec::with_capacity(attachments.len());
                for result in attachments {
                    let value = result?.await?;
                    ready_attachments.push(value);
                }
                Some(ready_attachments)
            }
        };

        let pdf = match results.1 {
            None => None,
            Some(url) => Some(url.await?)
        };

        Ok(PreviewResult {
            page_count: results.0,
            pages,
            attachments,
            pdf,
            signatures: results.4,
            protected: results.5,
        })
    }
}

impl PreviewService {
    fn validate_pages(&self, start_page_number: u16, end_page_number: u16, source_document: &PdfDocument) -> Result<(), &'static str> {
        if start_page_number > end_page_number {
            return Err("Start page number can't be greater than end page number.");
        }
        let pages = source_document.pages();
        if end_page_number > pages.len() {
            return Err("End page number exceeds pages of document.");
        }
        Ok(())
    }

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
