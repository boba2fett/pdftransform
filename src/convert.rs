use std::collections::BTreeMap;

use futures::StreamExt;
use bytes::Bytes;
use pdf::file::File;
use pdf::error::PdfError;

use crate::{persistence::{get_job_model, set_error}, models::SourceFile};

pub async fn process_job(job_id: String) {
    let job_model = get_job_model(&job_id).await.unwrap();
    let client = reqwest::Client::new();
    let ref_client = &client;
    let source_files = job_model.source_files;
    
    let source_files = futures::stream::iter(source_files)
    .map(|source_file| {
        let source_uri = source_file.source_uri.clone();
        async move {
            (source_file.source_file_id, dowload_source_file(ref_client, source_uri).await)
        }
    }).buffer_unordered(10).collect::<Vec<(String, Bytes)>>().await;

    for document in job_model.documents {
        for page in document.binaries {
            let source_file = source_files.iter().find(|(id, _)| *id == page.source_file_id).unwrap();
            //TODO rotate
            if 123 <= page.page_index {
                return set_error(&job_id).await.unwrap()
            }
            add_page("new_document".to_string(), "source_document".to_string(), page.page_index);
        }
    }
}

async fn dowload_source_file(client: &reqwest::Client, source_file_url: String) -> Bytes {
    let response = client.get(source_file_url).send().await;
    response.unwrap().bytes().await.unwrap()
}


fn add_page(new_document: String, source_document: String, page_index: usize) -> String {
    "asd".to_string()
}