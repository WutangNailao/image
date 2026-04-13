pub fn luminance_rgb(r: f64, g: f64, b: f64) -> f64 {
    0.299 * r + 0.587 * g + 0.114 * b
}

pub fn luminance_normalized_rgba(r: u8, g: u8, b: u8) -> f64 {
    luminance_rgb(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0)
}

pub fn normalized_rgba_channel(r: u8, g: u8, b: u8, a: u8, channel: i32) -> Option<f64> {
    match channel {
        0 => Some(r as f64 / 255.0),
        1 => Some(g as f64 / 255.0),
        2 => Some(b as f64 / 255.0),
        3 => Some(a as f64 / 255.0),
        4 => Some(luminance_normalized_rgba(r, g, b)),
        _ => None,
    }
}
