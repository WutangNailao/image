use crate::core::{ImageResult, error, input_rgba_image, success};
use crate::utils::color::normalized_rgba_channel;
use crate::utils::math::{clamp_to_u8, mix_f64};
use crate::utils::sampling::clamp_edges;

pub unsafe extern "C" fn image_convolution_rgba8_impl(
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
    if filter.is_null() {
        return error("filter is null");
    }

    let src = match unsafe { input_rgba_image(data, width, height, channels) } {
        Ok(image) => image,
        Err(message) => return error(message),
    };

    let filter = unsafe { std::slice::from_raw_parts(filter, 9) };
    let source = src.as_raw();
    let mut out = source.to_vec();
    let mask_source = if mask_data.is_null() {
        None
    } else {
        let mask =
            match unsafe { input_rgba_image(mask_data, mask_width, mask_height, mask_channels) } {
                Ok(image) => image,
                Err(message) => return error(message),
            };
        if mask_width < width || mask_height < height {
            return error("mask dimensions must cover source image");
        }
        if !(0..=4).contains(&mask_channel) {
            return error("unsupported mask channel");
        }
        Some(mask)
    };

    for y in 0..height {
        for x in 0..width {
            let mut r = 0.0f64;
            let mut g = 0.0f64;
            let mut b = 0.0f64;

            let mut fi = 0usize;
            for j in 0..3 {
                let yv = clamp_edges(y - 1 + j, height);
                for i in 0..3 {
                    let xv = clamp_edges(x - 1 + i, width);
                    let index = ((yv * width + xv) * 4) as usize;
                    let coefficient = filter[fi];
                    r += source[index] as f64 * coefficient;
                    g += source[index + 1] as f64 * coefficient;
                    b += source[index + 2] as f64 * coefficient;
                    fi += 1;
                }
            }

            let convolved_r = ((r / div) + offset).clamp(0.0, 255.0);
            let convolved_g = ((g / div) + offset).clamp(0.0, 255.0);
            let convolved_b = ((b / div) + offset).clamp(0.0, 255.0);

            let index = ((y * width + x) * 4) as usize;
            let source_r = source[index] as f64;
            let source_g = source[index + 1] as f64;
            let source_b = source[index + 2] as f64;
            let mx = if let Some(mask) = &mask_source {
                let mask_index = ((y * mask_width + x) * 4) as usize;
                let mask_amount = normalized_rgba_channel(
                    mask.as_raw()[mask_index],
                    mask.as_raw()[mask_index + 1],
                    mask.as_raw()[mask_index + 2],
                    mask.as_raw()[mask_index + 3],
                    mask_channel,
                )
                .unwrap_or(0.0);
                mask_amount * amount
            } else {
                amount
            };

            out[index] = clamp_to_u8(mix_f64(source_r, convolved_r, mx));
            out[index + 1] = clamp_to_u8(mix_f64(source_g, convolved_g, mx));
            out[index + 2] = clamp_to_u8(mix_f64(source_b, convolved_b, mx));
        }
    }

    success(out, width, height, 4)
}
