mod core;
mod filter;
mod transform;
mod utils;

pub use core::{ImageBuffer, ImageResult, image_free_buffer, image_last_error_message};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_resize_rgba8(
    data: *const u8,
    width: i32,
    height: i32,
    channels: i32,
    target_width: i32,
    target_height: i32,
    interpolation: i32,
) -> ImageResult {
    unsafe {
        transform::resize::image_resize_rgba8_impl(
            data,
            width,
            height,
            channels,
            target_width,
            target_height,
            interpolation,
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_crop_rgba8(
    data: *const u8,
    width: i32,
    height: i32,
    channels: i32,
    x: i32,
    y: i32,
    crop_width: i32,
    crop_height: i32,
) -> ImageResult {
    unsafe {
        transform::crop::image_crop_rgba8_impl(
            data,
            width,
            height,
            channels,
            x,
            y,
            crop_width,
            crop_height,
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_gaussian_blur_rgba8(
    data: *const u8,
    width: i32,
    height: i32,
    channels: i32,
    radius: i32,
) -> ImageResult {
    unsafe {
        filter::gaussian_blur::image_gaussian_blur_rgba8_impl(data, width, height, channels, radius)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_convolution_rgba8(
    data: *const u8,
    width: i32,
    height: i32,
    channels: i32,
    mask_data: *const u8,
    mask_width: i32,
    mask_height: i32,
    mask_channels: i32,
    mask_channel: i32,
    filter: *const f64,
    div: f64,
    offset: f64,
    amount: f64,
) -> ImageResult {
    unsafe {
        filter::convolution::image_convolution_rgba8_impl(
            data,
            width,
            height,
            channels,
            mask_data,
            mask_width,
            mask_height,
            mask_channels,
            mask_channel,
            filter,
            div,
            offset,
            amount,
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_separable_convolution_rgba8(
    data: *const u8,
    width: i32,
    height: i32,
    channels: i32,
    mask_data: *const u8,
    mask_width: i32,
    mask_height: i32,
    mask_channels: i32,
    mask_channel: i32,
    coefficients: *const f64,
    coefficient_count: i32,
) -> ImageResult {
    unsafe {
        filter::separable_convolution::image_separable_convolution_rgba8_impl(
            data,
            width,
            height,
            channels,
            mask_data,
            mask_width,
            mask_height,
            mask_channels,
            mask_channel,
            coefficients,
            coefficient_count,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_rgba(width: i32, height: i32) -> Vec<u8> {
        let mut out = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                out.extend_from_slice(&[x as u8 * 10, y as u8 * 20, 127, 255]);
            }
        }
        out
    }

    #[test]
    fn resize_returns_pixels() {
        let input = sample_rgba(2, 2);
        let result = unsafe { image_resize_rgba8(input.as_ptr(), 2, 2, 4, 4, 4, 0) };
        assert_eq!(result.code, core::IMAGE_OK);
        assert_eq!(result.buffer.width, 4);
        assert_eq!(result.buffer.height, 4);
        assert!(!result.buffer.data.is_null());
        unsafe {
            image_free_buffer(result.buffer.release_handle);
        }
    }

    #[test]
    fn crop_rejects_invalid_channels() {
        let input = sample_rgba(2, 2);
        let result = unsafe { image_crop_rgba8(input.as_ptr(), 2, 2, 3, 0, 0, 1, 1) };
        assert_eq!(result.code, core::IMAGE_ERROR);
        assert!(result.buffer.data.is_null());
    }

    #[test]
    fn gaussian_blur_returns_pixels() {
        let input = sample_rgba(4, 4);
        let result = unsafe { image_gaussian_blur_rgba8(input.as_ptr(), 4, 4, 4, 2) };
        assert_eq!(result.code, core::IMAGE_OK);
        assert_eq!(result.buffer.width, 4);
        assert_eq!(result.buffer.height, 4);
        assert!(!result.buffer.data.is_null());
        unsafe {
            image_free_buffer(result.buffer.release_handle);
        }
    }

    #[test]
    fn convolution_returns_pixels() {
        let input = sample_rgba(4, 4);
        let filter = [0.0, 1.0, 0.0, 1.0, -4.0, 1.0, 0.0, 1.0, 0.0];
        let result = unsafe {
            image_convolution_rgba8(
                input.as_ptr(),
                4,
                4,
                4,
                std::ptr::null(),
                0,
                0,
                0,
                -1,
                filter.as_ptr(),
                1.0,
                0.0,
                1.0,
            )
        };
        assert_eq!(result.code, core::IMAGE_OK);
        assert_eq!(result.buffer.width, 4);
        assert_eq!(result.buffer.height, 4);
        assert!(!result.buffer.data.is_null());
        unsafe {
            image_free_buffer(result.buffer.release_handle);
        }
    }

    #[test]
    fn convolution_with_mask_returns_pixels() {
        let input = sample_rgba(4, 4);
        let mask = sample_rgba(4, 4);
        let filter = [0.0, 1.0, 0.0, 1.0, -4.0, 1.0, 0.0, 1.0, 0.0];
        let result = unsafe {
            image_convolution_rgba8(
                input.as_ptr(),
                4,
                4,
                4,
                mask.as_ptr(),
                4,
                4,
                4,
                4,
                filter.as_ptr(),
                1.0,
                0.0,
                1.0,
            )
        };
        assert_eq!(result.code, core::IMAGE_OK);
        assert_eq!(result.buffer.width, 4);
        assert_eq!(result.buffer.height, 4);
        assert!(!result.buffer.data.is_null());
        unsafe {
            image_free_buffer(result.buffer.release_handle);
        }
    }

    #[test]
    fn separable_convolution_returns_pixels() {
        let input = sample_rgba(4, 4);
        let coefficients = [0.25, 0.5, 0.25];
        let result = unsafe {
            image_separable_convolution_rgba8(
                input.as_ptr(),
                4,
                4,
                4,
                std::ptr::null(),
                0,
                0,
                0,
                -1,
                coefficients.as_ptr(),
                coefficients.len() as i32,
            )
        };
        assert_eq!(result.code, core::IMAGE_OK);
        assert_eq!(result.buffer.width, 4);
        assert_eq!(result.buffer.height, 4);
        assert!(!result.buffer.data.is_null());
        unsafe {
            image_free_buffer(result.buffer.release_handle);
        }
    }

    #[test]
    fn separable_convolution_with_mask_returns_pixels() {
        let input = sample_rgba(4, 4);
        let mask = sample_rgba(4, 4);
        let coefficients = [0.25, 0.5, 0.25];
        let result = unsafe {
            image_separable_convolution_rgba8(
                input.as_ptr(),
                4,
                4,
                4,
                mask.as_ptr(),
                4,
                4,
                4,
                4,
                coefficients.as_ptr(),
                coefficients.len() as i32,
            )
        };
        assert_eq!(result.code, core::IMAGE_OK);
        assert_eq!(result.buffer.width, 4);
        assert_eq!(result.buffer.height, 4);
        assert!(!result.buffer.data.is_null());
        unsafe {
            image_free_buffer(result.buffer.release_handle);
        }
    }
}
