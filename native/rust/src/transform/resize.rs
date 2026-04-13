use crate::core::{ImageResult, error, input_rgba_image, success};
use image::imageops::{self, FilterType};

fn interpolation_filter(interpolation: i32) -> Result<FilterType, String> {
    match interpolation {
        0 => Ok(FilterType::Nearest),
        1 => Ok(FilterType::Triangle),
        value => Err(format!("unsupported interpolation value {value}")),
    }
}

pub unsafe extern "C" fn image_resize_rgba8_impl(
    data: *const u8,
    width: i32,
    height: i32,
    channels: i32,
    target_width: i32,
    target_height: i32,
    interpolation: i32,
) -> ImageResult {
    let filter = match interpolation_filter(interpolation) {
        Ok(filter) => filter,
        Err(message) => return error(message),
    };
    if target_width <= 0 || target_height <= 0 {
        return error("target dimensions must be positive");
    }

    let src = match unsafe { input_rgba_image(data, width, height, channels) } {
        Ok(image) => image,
        Err(message) => return error(message),
    };

    let resized = imageops::resize(&src, target_width as u32, target_height as u32, filter);
    success(resized.into_raw(), target_width, target_height, 4)
}
