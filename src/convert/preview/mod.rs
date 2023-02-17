use crate::models::PreviewResult;

use self::pdfium::get_pdf_preview;
pub mod pdfium;

pub async fn get_preview(job_id: &str, token: &str, source_file: Vec<u8>, content_type: &str) -> Result<PreviewResult, &'static str> {
    get_pdf_preview(job_id, token, source_file).await
}