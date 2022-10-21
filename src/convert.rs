use std::collections::BTreeMap;

use futures::StreamExt;
use bytes::Bytes;
use lopdf::Document;

use crate::{persistence::get_job_model, models::SourceFile};

pub async fn process_job(job_id: String) {
    let job_model = get_job_model(job_id).await.unwrap();
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
        let mut new_document = Document::with_version("1.5");
        for page in document.binaries {
            let source_file = source_files.iter().find(|(id, _)| *id == page.source_file_id).unwrap();
            //TODO rotate
            let source_document = Document::load_mem(&source_file.1).unwrap();
            if source_document.get_pages().len() > page.page_index {
                
            }
        }
    }
}

async fn dowload_source_file(client: &reqwest::Client, source_file_url: String) -> Bytes {
    let response = client.get(source_file_url).send().await;
    response.unwrap().bytes().await.unwrap()
}
