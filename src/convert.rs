use std::{collections::BTreeMap, path::PathBuf};

use futures::StreamExt;
use bytes::Bytes;
use tokio::io::copy;
use tempfile::Builder;

use crate::{persistence::{get_job_model, set_error}, models::SourceFile};

pub async fn process_job(job_id: String) {
    let job_model = get_job_model(&job_id).await.unwrap();
    let client = reqwest::Client::new();
    let ref_client = &client;
    let source_files = job_model.source_files;
    
    let tmp_dir = Builder::new().prefix(&job_id).tempdir().unwrap();
    let source_files = futures::stream::iter(source_files)
    .map(|source_file| {
        let path = tmp_dir.path().join(&source_file.source_file_id);
        async move {
            dowload_source_file(ref_client, &source_file.source_uri, path).await
        }
    }).buffer_unordered(10).collect::<Vec<PathBuf>>().await;

    // for document in job_model.documents {
    //     //let mut new_doc = ;
    //     for page in document.binaries {
    //         let source_path = &source_files.iter().find(|(id, _)| id.eq(&&page.source_file_id)).unwrap().1;
    //         //load &source_path
    //         if "pages".len() <= page.page_index {
    //             return set_error(&job_id).await.unwrap()
    //         }
    //         //add_page(&mut new_doc, from, page.page_range);
    //     }
    // }
}

async fn dowload_source_file(client: &reqwest::Client, source_file_url: &str, path: PathBuf) -> PathBuf {
    let response = client.get(source_file_url).send().await;
    let content = response.unwrap().text().await.unwrap();
    let mut file = tokio::fs::File::create(&path).await.unwrap();
    copy(&mut content.as_bytes(), &mut file).await.unwrap();
    path
}


// fn add_page(new_document: &mut Option<File<Vec<u8>>>, source_document: File<Vec<u8>>, page_index: u32) -> File<Vec<u8>> {
    
// }