use fast_image_resize::{images::Image, PixelType, Resizer, MulDiv};
use image::{ExtendedColorType, ImageEncoder};
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use tauri::http::{Request, Response, StatusCode};
use std::io::Cursor;
use std::path::PathBuf;
use std::error::Error;

fn resize_image_to_fit(buf: Vec<u8>, width: u32, height: u32) -> Result<Vec<u8>, Box<dyn Error>> {
    let img = image::load_from_memory(&buf)?;

    // 0. Resize ONLY if the image doesn't fit in specified dimensions.
    let width_ratio = width as f64 / img.width() as f64;
    let height_ratio = height as f64 / img.height() as f64;
    let scale_factor = width_ratio.min(height_ratio);

    if scale_factor >= 1.0 {
        return Ok(buf);
    }

    // 1. Create source image from byte buffer.
    let mut src_image = Image::from_vec_u8(
        img.width(),
        img.height(), 
        img.to_rgba8().into_raw(), 
        PixelType::U8x4
    )?;

    // 2. Multiply alpha to avoid having gray borders in the image.
    // RGBA -> Premultiplied RGBA
    let mul_div = MulDiv::default();
    mul_div.multiply_alpha_inplace(&mut src_image)?;

    // Calculate destination dimensions based on scale factor.
    let dst_width = ((img.width() as f64 * scale_factor) as u32).max(1);
    let dst_height = ((img.height() as f64 * scale_factor) as u32).max(1);

    // 3. Create destination image buffer to store the resized image later.
    let mut dst_image = Image::new(
        dst_width,
        dst_height,
        src_image.pixel_type()
    );

    // 4. Resize to requested size.
    let mut resizer = Resizer::new();
    resizer.resize(&src_image, &mut dst_image, None)?;

    // 5. Divide alpha back to standard RGBA.
    // Premultiplied RGBA -> RGBA
    mul_div.divide_alpha_inplace(&mut dst_image)?;

    // 6. Encode image into a PNG.
    let mut result_buf = Cursor::new(Vec::new());
    let encoder = PngEncoder::new_with_quality(
        &mut result_buf, 
        CompressionType::Fast, 
        FilterType::NoFilter
    );
    encoder.write_image(
        dst_image.buffer(), 
        dst_width, 
        dst_height, 
        ExtendedColorType::Rgba8
    )?;

    Ok(result_buf.into_inner())
}

pub async fn generate_image_response(request: Request<Vec<u8>>) -> Response<Vec<u8>> {
    let path_str = request.uri().path();
    
    // Decode the URL
    let decoded_path = urlencoding::decode(path_str)
        .map(|s| s.to_string())
        .unwrap_or_else(|_| path_str.to_string());

    let path = if cfg!(windows) && decoded_path.starts_with('/') {
        PathBuf::from(&decoded_path[1..])
    } else {
        PathBuf::from(&decoded_path)
    };

    let extension = path.extension().unwrap_or_default().to_string_lossy();
    let mime_type = mime_guess::from_ext(&extension).first_or_octet_stream();
    if mime_type.type_() != mime::IMAGE {
        return Response::builder()
            .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
            .body("File is not an image.".as_bytes().to_vec())
            .unwrap();
    }

    // Asynchronously read the file and return it as a Response.
    match tokio::fs::read(&path).await {
        Ok(data) => {
            // Resize the image if needed to ensure we do not transfer images that are too large.
            let img_res = resize_image_to_fit(data, 1920, 1080);
            return match img_res {
                // Image converted successfully.
                Ok(img) => {
                    let is_png = img.starts_with(&[0x89, 0x50, 0x4E, 0x47]);
                    let content_type = if is_png { "image/png".to_string() } else { mime_type.to_string() };
                    Response::builder()
                        .header("Content-Type", content_type)
                        .header("Access-Control-Allow-Origin", "*")
                        .body(img)
                        .unwrap()
                },
                // Resize failed, return error message from resizing.
                Err(e) => Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(e.to_string().into_bytes())
                    .unwrap()
            }
        }
        Err(_) => {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("File not found.".as_bytes().to_vec())
                .unwrap()
        }
    }
}