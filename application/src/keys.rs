#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Key {
    Space,
    Left,
    Right,
    Up,
    Down,
    Backspace,
    Delete,
    Insert,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    Escape,
    Enter,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}

impl Key {
    pub fn is_letter(self) -> bool {
        match self {
            Self::A
            | Self::B
            | Self::C
            | Self::D
            | Self::E
            | Self::F
            | Self::G
            | Self::H
            | Self::I
            | Self::J
            | Self::K
            | Self::L
            | Self::M
            | Self::N
            | Self::O
            | Self::P
            | Self::Q
            | Self::R
            | Self::S
            | Self::T
            | Self::U
            | Self::V
            | Self::W
            | Self::X
            | Self::Y
            | Self::Z => true,
            _ => false,
        }
    }

    pub fn is_number(self) -> bool {
        match self {
            Self::Num1
            | Self::Num2
            | Self::Num3
            | Self::Num4
            | Self::Num5
            | Self::Num6
            | Self::Num7
            | Self::Num8
            | Self::Num9 => true,
            _ => false,
        }
    }
}

impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Space => f.pad("Space"),
            Self::Left => f.pad("Left"),
            Self::Right => f.pad("Right"),
            Self::Up => f.pad("Up"),
            Self::Down => f.pad("Down"),
            Self::Backspace => f.pad("Backspace"),
            Self::Delete => f.pad("Delete"),
            Self::Insert => f.pad("Insert"),
            Self::Numpad0 => f.pad("Numpad 0"),
            Self::Numpad1 => f.pad("Numpad 1"),
            Self::Numpad2 => f.pad("Numpad 2"),
            Self::Numpad3 => f.pad("Numpad 3"),
            Self::Numpad4 => f.pad("Numpad 4"),
            Self::Numpad5 => f.pad("Numpad 5"),
            Self::Numpad6 => f.pad("Numpad 6"),
            Self::Numpad7 => f.pad("Numpad 7"),
            Self::Numpad8 => f.pad("Numpad 8"),
            Self::Numpad9 => f.pad("Numpad 9"),
            Self::Escape => f.pad("Escape"),
            Self::Enter => f.pad("Enter"),
            Self::F1 => f.pad("F1"),
            Self::F2 => f.pad("F2"),
            Self::F3 => f.pad("F3"),
            Self::F4 => f.pad("F4"),
            Self::F5 => f.pad("F5"),
            Self::F6 => f.pad("F6"),
            Self::F7 => f.pad("F7"),
            Self::F8 => f.pad("F8"),
            Self::F9 => f.pad("F9"),
            Self::F10 => f.pad("F10"),
            Self::F11 => f.pad("F11"),
            Self::F12 => f.pad("F12"),
            Self::Num0 => f.pad("0"),
            Self::Num1 => f.pad("1"),
            Self::Num2 => f.pad("2"),
            Self::Num3 => f.pad("3"),
            Self::Num4 => f.pad("4"),
            Self::Num5 => f.pad("5"),
            Self::Num6 => f.pad("6"),
            Self::Num7 => f.pad("7"),
            Self::Num8 => f.pad("8"),
            Self::Num9 => f.pad("9"),
            Self::A => f.pad("A"),
            Self::B => f.pad("B"),
            Self::C => f.pad("C"),
            Self::D => f.pad("D"),
            Self::E => f.pad("E"),
            Self::F => f.pad("F"),
            Self::G => f.pad("G"),
            Self::H => f.pad("H"),
            Self::I => f.pad("I"),
            Self::J => f.pad("J"),
            Self::K => f.pad("K"),
            Self::L => f.pad("L"),
            Self::M => f.pad("M"),
            Self::N => f.pad("N"),
            Self::O => f.pad("O"),
            Self::P => f.pad("P"),
            Self::Q => f.pad("Q"),
            Self::R => f.pad("R"),
            Self::S => f.pad("S"),
            Self::T => f.pad("T"),
            Self::U => f.pad("U"),
            Self::V => f.pad("V"),
            Self::W => f.pad("W"),
            Self::X => f.pad("X"),
            Self::Y => f.pad("Y"),
            Self::Z => f.pad("Z"),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct Hotkey {
    pub key: Key,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Hotkey {
    pub fn new(key: Key) -> Self {
        Self {
            key,
            ctrl: false,
            alt: false,
            shift: false,
        }
    }

    pub fn ctrl(key: Key) -> Self {
        Self {
            key,
            ctrl: true,
            alt: false,
            shift: false,
        }
    }

    pub fn alt(key: Key) -> Self {
        Self {
            key,
            ctrl: false,
            alt: true,
            shift: false,
        }
    }

    pub fn shift(key: Key) -> Self {
        Self {
            key,
            ctrl: false,
            alt: false,
            shift: true,
        }
    }

    pub fn ctrl_shift(key: Key) -> Self {
        Self {
            key,
            ctrl: true,
            alt: false,
            shift: true,
        }
    }

    pub fn no_modifiers(&self) -> bool {
        !(self.ctrl || self.alt || self.shift)
    }
}

impl std::fmt::Debug for Hotkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad("[")?;
        if self.ctrl {
            f.pad("Ctrl+")?;
        }
        if self.alt {
            f.pad("Alt+")?;
        }
        if self.shift {
            f.pad("Shift+")?;
        }
        self.key.fmt(f)?;
        f.pad("]")
    }
}
