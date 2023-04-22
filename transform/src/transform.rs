use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
    time::Duration,
};

use common::download::DownloadedSourceFile;
use common::{
    models::{Document, Part, Rotation, TransformDocumentResult},
    persistence::{files::IFileStorage, tempfiles::TempJobFileProvider},
    util::routes::file_route,
};
use mime::Mime;
use pdfium_render::prelude::*;
use tracing::info;
use wait_timeout::ChildExt;

pub fn init_pdfium() -> Result<Pdfium, &'static str> {
    Ok(Pdfium::new(Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")).map_err(|_| "Could not init pdfium")?))
}

const LIBRE: &str = "/usr/bin/soffice";

pub fn check_libre() -> bool {
    Path::new("/usr/lib/libreoffice/program").exists()
}

#[async_trait::async_trait]
pub trait ITransformService: Send + Sync {
    async fn get_transformation<'a>(
        &self, job_id: &str, token: &str, documents: &Vec<Document>, source_files: Vec<&DownloadedSourceFile>, job_files: &TempJobFileProvider,
    ) -> Result<Vec<TransformDocumentResult>, &'static str>;
}

pub struct TransformService {
    pub storage: Arc<dyn IFileStorage>,
    pub pdfium: Pdfium,
}

#[async_trait::async_trait]
impl ITransformService for TransformService {
    async fn get_transformation<'a>(
        &self, job_id: &str, token: &str, documents: &Vec<Document>, source_files: Vec<&DownloadedSourceFile>, job_files: &TempJobFileProvider,
    ) -> Result<Vec<TransformDocumentResult>, &'static str> {
        let results: Vec<_> = {
            let mut cache: Option<(&str, PdfDocument)> = None;

            documents
                .iter()
                .map(|document| -> Result<_, &'static str> {
                    let cache_ref: &mut Option<(&str, PdfDocument)> = &mut cache;
                    let bytes = {
                        let mut new_doc = self.pdfium.create_new_pdf().map_err(|_| "Could not create document.")?;
                        for part in &document.parts {
                            if cache_ref.is_some() && cache_ref.as_ref().unwrap().0.eq(&part.source_file) {
                                self.add_part(&mut new_doc, &cache_ref.as_ref().unwrap().1, part)?;
                            } else {
                                let source_file = source_files.iter().find(|source_file| source_file.id.eq(&part.source_file)).ok_or("Could not find corresponding source file.")?;
                                if self.is_supported_image(&source_file.content_type) {
                                    self.add_image(&mut new_doc, &source_file, &part)?;
                                } else {
                                    let source_doc = if source_file.content_type != mime::APPLICATION_PDF && check_libre() {
                                        let source_path = self.load_from_libre(&source_file.path, &source_file.content_type, job_files)?;
                                        info!("Converted {} from libre", source_path.display());
                                        self.pdfium.load_pdf_from_file(&source_path, None).map_err(|_| "Could not create document.")?
                                    } else {
                                        self.pdfium.load_pdf_from_file(&source_file.path, None).map_err(|_| "Could not create document.")?
                                    };
                                    info!("source {} has {} pages", &source_file.id, source_doc.pages().len());
                                    *cache_ref = Some((&part.source_file, source_doc));
                                    self.add_part(&mut new_doc, &cache_ref.as_ref().unwrap().1, part)?;
                                }
                                info!("generated {} has {} pages", &document.id, new_doc.pages().len());
                            }
                        }
                        for attachment in &document.attachments {
                            let source_file = source_files.iter().find(|source_file| source_file.id.eq(&attachment.source_file)).ok_or("Could not find corresponding source file.")?;
                            new_doc.attachments_mut().create_attachment_from_file(&attachment.name, &source_file.path).map_err(|_| "Could not add attachment.")?;
                        }
                        new_doc.save_to_bytes().map_err(|_| "Could not save file.")?
                    };
                    Ok(async move {
                        info!("generated {} is {} KiB", &document.id, bytes.len() / 1024);
                        let file_id = self.storage.store_result_file(&job_id, &token, &document.id, Some("application/pdf"), bytes).await?;

                        Ok::<TransformDocumentResult, &'static str>(TransformDocumentResult {
                            download_url: file_route(&file_id, token),
                            id: document.id.to_string(),
                        })
                    })
                })
                .collect()
        };
        let mut document_results = Vec::with_capacity(documents.len());
        for result in results {
            let value = result?.await?;
            document_results.push(value);
        }
        Ok(document_results)
    }
}

impl TransformService {
    fn add_part(&self, new_document: &mut PdfDocument, source_document: &PdfDocument, part: &Part) -> Result<(), &'static str> {
        let start_page_number = part.start_page_number.unwrap_or(1);
        let end_page_number = part.end_page_number.unwrap_or(source_document.pages().len());
        self.validate_pages(start_page_number, end_page_number, source_document)?;

        let new_start_page_number = new_document.pages().len() + 1;
        let new_end_page_number = new_start_page_number + (end_page_number - start_page_number);

        new_document
            .pages()
            .copy_page_range_from_document(&source_document, start_page_number - 1..=end_page_number - 1, new_start_page_number - 1)
            .map_err(|_| "Could not transfer pages.")?;

        self.turn_pages(new_start_page_number, new_end_page_number, new_document, part)?;

        Ok(())
    }

    fn add_image(&self, new_document: &mut PdfDocument, source_file: &DownloadedSourceFile, part: &Part) -> Result<(), &'static str> {
        let source_img = image::io::Reader::open(&source_file.path).map_err(|_| "")?.with_guessed_format().map_err(|_| "")?.decode().map_err(|_| "")?;

        let source_img = {
            match &part.rotation {
                Some(Rotation::N270) => source_img.rotate90(),
                Some(Rotation::N180) => source_img.rotate180(),
                Some(Rotation::N90) => source_img.rotate270(),
                Some(Rotation::P90) => source_img.rotate90(),
                Some(Rotation::P180) => source_img.rotate180(),
                Some(Rotation::P270) => source_img.rotate270(),
                _ => source_img,
            }
        };

        let object = PdfPageImageObject::new_with_width(&new_document, &source_img, PdfPoints::new(source_img.width() as f32)).map_err(|_| "")?;

        let mut page = new_document
            .pages()
            .create_page_at_end(PdfPagePaperSize::Custom(PdfPoints::new(source_img.width() as f32), PdfPoints::new(source_img.height() as f32)))
            .map_err(|_| "")?;
        page.objects_mut().add_image_object(object).map_err(|_| "")?;
        Ok(())
    }

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

    fn turn_pages(&self, start_page_number: u16, end_page_number: u16, source_document: &PdfDocument, part: &Part) -> Result<(), &'static str> {
        if part.rotation.is_some() {
            let part_rotation: i32 = part.rotation.unwrap_or(Rotation::P0).as_degrees();
            let part_rotation: i32 = {
                if part_rotation < 0 {
                    360 + part_rotation
                } else {
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

    fn is_supported_image(&self, content_type: &Mime) -> bool {
        content_type.eq(&mime::IMAGE_PNG) || content_type.eq(&mime::IMAGE_JPEG) || content_type.eq(&mime::IMAGE_GIF) || content_type.eq(&mime::IMAGE_BMP)
    }

    fn load_from_libre(&self, source: &PathBuf, source_mime_type: &Mime, job_files: &TempJobFileProvider) -> Result<PathBuf, &'static str> {
        let result_path = job_files.get_path();
        std::fs::create_dir_all(&result_path).unwrap();
        let output = &result_path.as_os_str();
        let result_path = result_path.join(source.file_name().unwrap()).with_extension("pdf");
        let input = source.as_os_str();

        let mut child = Command::new(LIBRE)
            .arg("--headless")
            .arg("--convert-to")
            .arg("pdf")
            .arg(input)
            .arg("--outdir")
            .arg(output)
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|_| "Could not start libre")?;

        let one_sec = Duration::from_secs(30);
        let status_code = match child.wait_timeout(one_sec).map_err(|_| "Could not wait on libre")? {
            Some(status) => status.code(),
            None => {
                child.kill().map_err(|_| "Could not kill libre")?;
                child.wait().map_err(|_| "Could not wait for libre")?.code()
            }
        };
        match status_code {
            Some(0) => Ok(result_path),
            _ => {
                info!("Libre failed with '{:?}'", child.stdout);
                Err("Something went wrong")
            }
        }
    }
}
