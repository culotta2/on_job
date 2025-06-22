pub enum TextEffect {
    StrikeThrough,
    Red,
    Green,
}

impl TextEffect {
    pub fn ansi_code(&self) -> &str {
        match *self {
            TextEffect::StrikeThrough => "9",
            TextEffect::Red => "31",
            TextEffect::Green => "32",
        }
    }
}

impl std::fmt::Display for TextEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.ansi_code())
    }
}

pub fn add_text_effect(text: &str, effect_str: TextEffect) -> String {
    format!("\x1b[{}m{}\x1b[0m", effect_str, text)
}
