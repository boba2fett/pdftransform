pub fn preview_job_route(job_id: &str, token: &str) -> String {
    format!("/preview/{}?token={}", &job_id, token)
}

pub fn transform_job_route(job_id: &str, token: &str) -> String {
    format!("/transform/{}?token={}", &job_id, token)
}

pub fn file_route(file_id: &str, token: &str) -> String {
    format!("/file/{}?token={}", file_id, token)
}
