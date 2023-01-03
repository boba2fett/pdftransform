use actix_web::http::header::ContentType;

pub fn is_supported_image(content_type: &ContentType) -> bool {
    content_type.is_png()
      || content_type.is_jpeg()
      || content_type.is_gif()
      || content_type.is_icon()
      || content_type.is_bmp()
      //|| content_type.is_tiff()
}