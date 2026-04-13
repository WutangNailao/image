pub fn reflect_index(max: i32, value: i32) -> i32 {
    let reflected = if value < 0 {
        -value
    } else if value >= max {
        max - (value - max) - 1
    } else {
        value
    };
    reflected.clamp(0, max.saturating_sub(1))
}

pub fn clamp_edges(value: i32, max: i32) -> i32 {
    value.clamp(0, max.saturating_sub(1))
}
