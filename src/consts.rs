use pdfium_render::prelude::Pdfium;

pub static VERSION: &str = env!("CARGO_PKG_VERSION");
pub static NAME: &str = env!("CARGO_PKG_NAME");
pub static mut PARALLELISM: usize = 10;
pub static mut PDFIUM: Option<Pdfium> = None;
pub static mut MONGO_CLIENT: Option<mongodb::Client> = None;
