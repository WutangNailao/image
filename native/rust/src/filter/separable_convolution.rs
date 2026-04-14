use crate::core::{ImageResult, error, input_rgba_image, success};
use crate::utils::color::normalized_rgba_channel;
use crate::utils::sampling::reflect_index;

fn apply_line(
    src: &[u8],
    dst: &mut [u8],
    width: i32,
    height: i32,
    coefficients: &[f64],
    radius: i32,
    horizontal: bool,
    mask_source: Option<&[u8]>,
    mask_width: i32,
    mask_channel: i32,
) {
    let line_count = if horizontal { height } else { width };
    let line_width = if horizontal { width } else { height };

    for y in 0..line_count {
        for x in 0..line_width {
            let mut accum = [0.0f64; 4];

            for (offset, coefficient) in (-radius..=radius).zip(coefficients.iter().copied()) {
                let reflected = reflect_index(line_width, x + offset);
                let (sample_x, sample_y) = if horizontal {
                    (reflected, y)
                } else {
                    (y, reflected)
                };
                let sample_index = ((sample_y * width + sample_x) * 4) as usize;
                for channel in 0..4 {
                    accum[channel] += coefficient * src[sample_index + channel] as f64;
                }
            }

            let (dest_x, dest_y) = if horizontal { (x, y) } else { (y, x) };
            let dest_index = ((dest_y * width + dest_x) * 4) as usize;

            if let Some(mask) = mask_source {
                let mask_index = ((dest_y * mask_width + dest_x) * 4) as usize;
                let amount = normalized_rgba_channel(
                    mask[mask_index],
                    mask[mask_index + 1],
                    mask[mask_index + 2],
                    mask[mask_index + 3],
                    mask_channel,
                )
                .unwrap_or(0.0);

                for channel in 0..4 {
                    let current = dst[dest_index + channel] as f64;
                    let filtered = accum[channel];
                    let mix_amount = amount;
                    let left = current * (1.0 - mix_amount);
                    let right = filtered * mix_amount;
                    dst[dest_index + channel] = (left + right).clamp(0.0, 255.0) as u8;
                }
            } else {
                for channel in 0..4 {
                    dst[dest_index + channel] = accum[channel].clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
}

pub unsafe extern "C" fn image_separable_convolution_rgba8_impl(
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
    if coefficients.is_null() {
        return error("coefficients is null");
    }
    if coefficient_count <= 0 || coefficient_count % 2 == 0 {
        return error("coefficient count must be a positive odd number");
    }

    let src = match unsafe { input_rgba_image(data, width, height, channels) } {
        Ok(image) => image,
        Err(message) => return error(message),
    };

    let coefficients =
        unsafe { std::slice::from_raw_parts(coefficients, coefficient_count as usize) };
    let radius = coefficient_count / 2;
    let source = src.as_raw();
    let mut tmp = source.to_vec();
    let mut out = source.to_vec();

    let mask_image = if mask_data.is_null() {
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
    let mask_source = mask_image.as_ref().map(|mask| mask.as_raw());

    apply_line(
        source,
        &mut tmp,
        width,
        height,
        coefficients,
        radius,
        true,
        mask_source.map(|mask| &mask[..]),
        mask_width,
        mask_channel,
    );
    apply_line(
        &tmp,
        &mut out,
        width,
        height,
        coefficients,
        radius,
        false,
        mask_source.map(|mask| &mask[..]),
        mask_width,
        mask_channel,
    );

    success(out, width, height, 4)
}
