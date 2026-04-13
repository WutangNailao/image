pub fn clamp_to_u8(value: f64) -> u8 {
    value.clamp(0.0, 255.0) as u8
}

pub fn mix_f64(source: f64, target: f64, amount: f64) -> f64 {
    source * (1.0 - amount) + target * amount
}
