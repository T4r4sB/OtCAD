use application::clipboard::*;
use application::font::*;
use application::gui::gui_components::*;
use application::gui::*;

pub fn create_default_size_button(text: &str, font: Font) -> Button {
    let text_size = font.get_size(text);
    Button::new(
        SizeConstraints(
            SizeConstraint::fixed((text_size.0 + text_size.1) as i32),
            SizeConstraint::fixed(text_size.1 as i32 + 2),
        ),
        text.to_string(),
        font,
    )
}

pub fn create_default_size_check_button(text: &str, font: Font) -> Button {
    let text_size = font.get_size(text);
    let check_width = font.get_size("V").0 as i32;
    Button::new(
        SizeConstraints(
            SizeConstraint::fixed((text_size.0 + text_size.1) as i32 + check_width),
            SizeConstraint::fixed(text_size.1 as i32 + 2),
        ),
        text.to_string(),
        font,
    )
}

pub fn create_default_size_edit(text: &str, font: Font, clipboard: Clipboard) -> Edit {
    let text_size = font.get_size(text);
    Edit::new(
        SizeConstraints(
            SizeConstraint::fixed((text_size.0 + text_size.1) as i32),
            SizeConstraint::fixed(text_size.1 as i32 + 2),
        ),
        font,
        clipboard,
    )
    .text(text)
}

pub fn create_default_size_text_box(text: &str, font: Font) -> TextBox {
    let text_size = font.get_size(text);
    TextBox::new(
        SizeConstraints(
            SizeConstraint::fixed((text_size.0 + text_size.1) as i32),
            SizeConstraint::fixed(text_size.1 as i32 + 2),
        ),
        text.to_string(),
        font,
    )
}
