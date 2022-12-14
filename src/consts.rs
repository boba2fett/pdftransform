use pdfium_render::prelude::Pdfium;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub static mut PARALLELISM: usize = 10;
pub static mut MAX_KIBIBYTES: usize = 4096;
pub static mut PDFIUM: Option<Pdfium> = None;
