use crate::core::{ImageResult, error, input_rgba_image, success};
use image::imageops;

pub unsafe extern "C" fn image_crop_rgba8_impl(
    data: *const u8,
    width: i32,
    height: i32,
    channels: i32,
    x: i32,
    y: i32,
    crop_width: i32,
    crop_height: i32,
) -> ImageResult {
    if crop_width <= 0 || crop_height <= 0 {
        return error("crop dimensions must be positive");
    }

    let src = match unsafe { input_rgba_image(data, width, height, channels) } {
        Ok(image) => image,
        Err(message) => return error(message),
    };

    let max_width = width.saturating_sub(x).max(0);
    let max_height = height.saturating_sub(y).max(0);
    let bounded_width = crop_width.min(max_width);
    let bounded_height = crop_height.min(max_height);

    if x < 0 || y < 0 || bounded_width <= 0 || bounded_height <= 0 {
        return error("crop rectangle is outside image bounds");
    }

    let cropped = imageops::crop_imm(
        &src,
        x as u32,
        y as u32,
        bounded_width as u32,
        bounded_height as u32,
    )
    .to_image();

    success(cropped.into_raw(), bounded_width, bounded_height, 4)
}
