use std::path::PathBuf;
use futures::StreamExt;
use tokio::io::copy;
use tempfile::Builder;

use crate::{persistence::{get_job_model, set_ready}, models::DocumentResult, transform::{add_page, get_pdfium}};

pub async fn process_job(job_id: String) -> () {
    let job_model = get_job_model(&job_id).await.unwrap();
    let client = reqwest::Client::new();
    let ref_client = &client;
    //let source_files = job_model.source_files;
    
    let tmp_dir = Builder::new().prefix(&job_id).tempdir().unwrap();

    let source_files = futures::stream::iter(job_model.source_files)
    .map(|source_file| {
        let path = tmp_dir.path().join(&source_file.source_file_id);
        async move {
            dowload_source_file(ref_client, &source_file.source_uri, path).await
        }
    }).buffer_unordered(10).collect::<Vec<PathBuf>>().await;

    dbg!(&source_files);

    // let source_file_handles = source_files.iter().map(|source_file| {
    //     let path = tmp_dir.path().join(&source_file.source_file_id);
    //     tokio::spawn(async {
    //         dowload_source_file(ref_client, &source_file.source_uri, path).await
    //     })
    // }).collect::<Vec<tokio::task::JoinHandle<PathBuf>>>();
    // let source_file_results: Vec<Result<PathBuf, JoinError>> = futures::future::join_all(source_file_handles).await;
    // let source_files: Vec<PathBuf> = source_file_results.into_iter().flatten().collect();
    // let mut source_files: Vec<PathBuf> = Vec::with_capacity(job_model.source_files.len());
    // for source_file in job_model.source_files {
    //     let path = tmp_dir.path().join(&source_file.source_file_id);
    //     source_files.push(dowload_source_file(ref_client, &source_file.source_uri, path).await);
    // }
    

    let results = {
        let mut results = Vec::with_capacity(job_model.documents.len());
        let pdfium = get_pdfium();
        for document in job_model.documents {
            let mut new_doc = pdfium.create_new_pdf().unwrap();
            for part in document.binaries {
                let source_path = source_files.iter().find(|path| path.ends_with(&part.source_file_id)).unwrap();
                let source_doc = pdfium.load_pdf_from_file(source_path, None).unwrap();
                // if source_doc.pages().len() <= part.start_page_number.unwrap_or_else(|| u16::MIN) || source_doc.pages().len() <= part.end_page_number.unwrap_or_else(|| u16::MAX) {
                //     // _ = set_error(&job_id).await;
                //     //return ()
                //     panic!("asd");
                // }
                add_page(&mut new_doc, source_doc, &part);
            }
            let path = tmp_dir.path().join(&document.id);
            new_doc.save_to_file(&path).unwrap();

            results.push(DocumentResult {
                download_url: format!("/convert/{}/{}", &job_id, document.id),
                id: document.id.to_string(),
            });
        }
        results
    };

    set_ready(&job_id, results).await.unwrap();
    ()
}

async fn dowload_source_file(client: &reqwest::Client, source_file_url: &str, path: PathBuf) -> PathBuf {
    let response = client.get(source_file_url).send().await;
    let content = response.unwrap().text().await.unwrap();
    let mut file = tokio::fs::File::create(&path).await.unwrap();
    copy(&mut content.as_bytes(), &mut file).await.unwrap();
    path
}


// {
//     "callbackUri": "a",
//     "documents": [{"id": "d1", "binaries": [
//      {"sourceFileId": "s1", "rotation": 0}
//   ]}],
//     "sourceFiles": [{"sourceFileId": "s1",
//                      "sourceUri": "https://courses.cs.washington.edu/courses/cse571/16au/slides/10-sdf.pdf"}]
//   }