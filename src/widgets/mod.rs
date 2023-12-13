use ratatui::style::Color;

pub mod card_info;
pub mod dependencies;
pub mod enum_choice;
pub mod file_finder;
pub mod table_thing;

pub fn _to_color(value: String) -> Color {
    let mut r = 0;
    let mut g = 0;
    let mut b = 0;

    for (byte, i) in value.as_bytes().iter().enumerate() {
        match i % 3 {
            0 => r += byte,
            1 => g += byte,
            2 => b += byte,
            _ => unreachable!(),
        }
    }

    let r = (r % 256) as u8;
    let g = (g % 256) as u8;
    let b = (b % 256) as u8;

    Color::Rgb(r, g, b)
}
