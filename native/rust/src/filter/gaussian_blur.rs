use crate::core::{ImageResult, error, input_rgba_image, success};
use crate::utils::sampling::reflect_index;

fn gaussian_kernel(radius: i32) -> Vec<f32> {
    let sigma = radius as f32 * (2.0 / 3.0);
    let s = 2.0 * sigma * sigma;
    let mut kernel = Vec::with_capacity((radius * 2 + 1) as usize);
    let mut sum = 0.0f32;

    for x in -radius..=radius {
        let coefficient = (-((x * x) as f32) / s).exp();
        sum += coefficient;
        kernel.push(coefficient);
    }

    if sum != 0.0 {
        for coefficient in &mut kernel {
            *coefficient /= sum;
        }
    }

    kernel
}

fn convolve_line(
    src: &[u8],
    dst: &mut [u8],
    width: i32,
    height: i32,
    kernel: &[f32],
    radius: i32,
    horizontal: bool,
) {
    let line_count = if horizontal { height } else { width };
    let line_width = if horizontal { width } else { height };

    for y in 0..line_count {
        for x in 0..line_width {
            let mut accum = [0.0f32; 4];

            for (offset, coefficient) in (-radius..=radius).zip(kernel.iter().copied()) {
                let reflected = reflect_index(line_width, x + offset);
                let (sample_x, sample_y) = if horizontal {
                    (reflected, y)
                } else {
                    (y, reflected)
                };
                let sample_index = ((sample_y * width + sample_x) * 4) as usize;
                for channel in 0..4 {
                    accum[channel] += coefficient * src[sample_index + channel] as f32;
                }
            }

            let (dest_x, dest_y) = if horizontal { (x, y) } else { (y, x) };
            let dest_index = ((dest_y * width + dest_x) * 4) as usize;
            for channel in 0..4 {
                dst[dest_index + channel] = accum[channel].clamp(0.0, 255.0) as u8;
            }
        }
    }
}

pub unsafe extern "C" fn image_gaussian_blur_rgba8_impl(
    data: *const u8,
    width: i32,
    height: i32,
    channels: i32,
    radius: i32,
) -> ImageResult {
    if radius <= 0 {
        return error("radius must be positive");
    }

    let src = match unsafe { input_rgba_image(data, width, height, channels) } {
        Ok(image) => image,
        Err(message) => return error(message),
    };

    let kernel = gaussian_kernel(radius);
    let source = src.as_raw();
    let mut tmp = vec![0u8; source.len()];
    let mut out = vec![0u8; source.len()];

    convolve_line(source, &mut tmp, width, height, &kernel, radius, true);
    convolve_line(&tmp, &mut out, width, height, &kernel, radius, false);

    success(out, width, height, 4)
}
