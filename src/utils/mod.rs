use std::cmp::Ordering;

pub fn right_pad(input: &str, length: usize, padding_char: char) -> String {
    match input.len().cmp(&length) {
        Ordering::Less => input.to_owned() + &padding_char.to_string().repeat(length - input.len()),
        _ => input.into(),
    }
}
