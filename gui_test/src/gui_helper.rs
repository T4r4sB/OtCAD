use application::clipboard::*;
use application::font::*;
use application::gui::gui_components::*;
use application::gui::*;
use application::keys::*;

pub fn create_default_size_button(text: &str, font: Font) -> Button {
    Button::new(
        Button::default_size(text, None, &font),
        text.to_string(),
        font,
    )
}

pub fn create_default_size_button_with_hotkey(
    text: &str,
    font: Font,
    hotkey: Hotkey,
    global: bool,
) -> Button {
    Button::new(
        Button::default_size(text, Some(hotkey), &font),
        text.to_string(),
        font,
    )
    .hotkey(hotkey, global)
}

pub fn create_default_size_check_button(text: &str, font: Font) -> Button {
    Button::new(
        Button::default_checkbox_size(text, None, &font),
        text.to_string(),
        font,
    )
}

pub fn create_default_size_radio_button(text: &str, font: Font) -> Button {
    Button::new(
        Button::default_radiobutton_size(text, None, &font),
        text.to_string(),
        font,
    )
}

pub fn create_default_size_edit(text: &str, font: Font, clipboard: Clipboard) -> Edit {
    Edit::new(GuiSystem::default_size(text, None, &font), font, clipboard).text(text)
}

pub fn create_default_size_text_box(text: &str, font: Font) -> TextBox {
    TextBox::new(
        GuiSystem::default_size(text, None, &font),
        text.to_string(),
        font,
    )
}
