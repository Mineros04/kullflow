use fast_image_resize::{MulDiv, PixelType, Resizer, images::Image};
use std::error::Error;

pub fn resize_image_to_fit(
    buf: Vec<u8>,
    width: u32,
    height: u32
) -> Result<(Vec<u8>, u32, u32), Box<dyn Error>> {
    let img = image::load_from_memory(&buf)?;

    // 0. Resize ONLY if the image doesn't fit in specified dimensions.
    let width_ratio = width as f64 / img.width() as f64;
    let height_ratio = height as f64 / img.height() as f64;
    let scale_factor = width_ratio.min(height_ratio);

    if scale_factor >= 1.0 {
        return Ok((buf, img.width(), img.height()));
    }

    // 1. Create source image from byte buffer.
    let mut src_image =
        Image::from_vec_u8(img.width(), img.height(), img.to_rgba8().into_raw(), PixelType::U8x4)?;

    // 2. Multiply alpha to avoid having gray borders in the image.
    // RGBA -> Premultiplied RGBA
    let mul_div = MulDiv::default();
    mul_div.multiply_alpha_inplace(&mut src_image)?;

    // Calculate destination dimensions based on scale factor.
    let dst_width = ((img.width() as f64 * scale_factor) as u32).max(1);
    let dst_height = ((img.height() as f64 * scale_factor) as u32).max(1);

    // 3. Create destination image buffer to store the resized image later.
    let mut dst_image = Image::new(dst_width, dst_height, src_image.pixel_type());

    // 4. Resize to requested size.
    let mut resizer = Resizer::new();
    resizer.resize(&src_image, &mut dst_image, None)?;

    // 5. Divide alpha back to standard RGBA.
    // Premultiplied RGBA -> RGBA
    mul_div.divide_alpha_inplace(&mut dst_image)?;

    Ok((dst_image.into_vec(), dst_width, dst_height))
}
