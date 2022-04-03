use crate::{errors::WmResult, wm::actions::Action};

#[derive(Debug)]
pub struct Keybinds(Vec<Keybind>);

impl Default for Keybinds {
    fn default() -> Self {
        let default_binds = vec![
            Keybind::new(vec![Key::Mod, Key::KeyQ], Action::Kill),
            Keybind::new(
                vec![Key::Mod, Key::KeyL],
                Action::Focus("left".try_into().unwrap()),
            ),
            Keybind::new(
                vec![Key::Mod, Key::KeyH],
                Action::Focus("right".try_into().unwrap()),
            ),
            Keybind::new(
                vec![Key::Mod, Key::KeyJ],
                Action::Focus("down".try_into().unwrap()),
            ),
            Keybind::new(
                vec![Key::Mod, Key::KeyK],
                Action::Focus("up".try_into().unwrap()),
            ),
            Keybind::new(vec![Key::Mod, Key::Enter], Action::Execute("xterm".into())),
        ];
        Self(default_binds)
    }
}

impl Keybinds {
    pub fn add(&mut self, keys: String, action: String) -> WmResult {
        let keybind = Keybind::from(keys, action)?;
        self.0.push(keybind);
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[allow(unused)]
pub enum Key {
    Noop = 0x00,
    Esc = 0x01,
    F1 = 0x02,
    F2 = 0x03,
    F3 = 0x04,
    F4 = 0x05,
    F5 = 0x06,
    F6 = 0x07,
    F7 = 0x08,
    F8 = 0x09,
    F9 = 0x0a,
    F10 = 0x0b,
    F11 = 0x0c,
    F12 = 0x0d,
    Print = 0x0e,
    ScrollLock = 0x0f,
    Pause = 0x10,
    Backtick = 0x11,
    Key1 = 0x12,
    Key2 = 0x13,
    Key3 = 0x14,
    Key4 = 0x15,
    Key5 = 0x16,
    Key6 = 0x17,
    Key7 = 0x18,
    Key8 = 0x19,
    Key9 = 0x1a,
    Key0 = 0x1b,
    Minus = 0x1c,
    Equals = 0x1d,
    Backspace = 0x1e,
    Insert = 0x1f,
    Home = 0x20,
    Prior = 0x21,
    NumLock = 0x22,
    NumDivide = 0x23,
    NumMultiply = 0x24,
    NumSubstract = 0x25,
    Tab = 0x26,
    KeyQ = 0x27,
    KeyW = 0x28,
    KeyE = 0x29,
    KeyR = 0x2a,
    KeyT = 0x2b,
    KeyY = 0x2c,
    KeyU = 0x2d,
    KeyI = 0x2e,
    KeyO = 0x2f,
    KeyP = 0x30,
    RightAngleBracket = 0x31,
    LeftAngleBracket = 0x32,
    Backslash = 0x33,
    Delete = 0x34,
    End = 0x35,
    Next = 0x36,
    Num7 = 0x37,
    Num8 = 0x38,
    Num9 = 0x39,
    NumAdd = 0x3a,
    CapsLock = 0x3b,
    KeyA = 0x3c,
    KeyS = 0x3d,
    KeyD = 0x3e,
    KeyF = 0x3f,
    KeyG = 0x40,
    KeyH = 0x41,
    KeyJ = 0x42,
    KeyK = 0x43,
    KeyL = 0x44,
    Semicolon = 0x45,
    Quote = 0x46,
    Enter = 0x47,
    Num4 = 0x48,
    Num5 = 0x49,
    Num6 = 0x4a,
    LShift = 0x4b,
    KeyZ = 0x4c,
    KeyX = 0x4d,
    KeyC = 0x4e,
    KeyV = 0x4f,
    KeyB = 0x50,
    KeyN = 0x51,
    KeyM = 0x52,
    Colon = 0x53,
    Period = 0x54,
    Slash = 0x55,
    RShift = 0x56,
    Up = 0x57,
    Num1 = 0x58,
    Num2 = 0x59,
    Num3 = 0x5a,
    NumEnter = 0x5b,
    Ctrl = 0x5c,
    Mod = 0x66,
    Alt = 0x5d,
    Space = 0x5e,
    RAlt = 0x5f,
    Fn = 0x67,
    Menu = 0x68,
    RCtrl = 0x60,
    Down = 0x61,
    Left = 0x62,
    Right = 0x63,
    Num0 = 0x64,
    NumDecimal = 0x65,
}

impl TryFrom<u8> for Key {
    type Error = crate::errors::Error;

    fn try_from(c: u8) -> Result<Self, Self::Error> {
        if c > 0x68 {
            Err("conversion error: unable to convert keycode to Key".into())
        } else {
            unsafe { Ok(std::mem::transmute(c)) }
        }
    }
}

impl Key {
    fn from_vec(input: &Vec<String>) -> WmResult<Vec<Self>> {
        let mut ret = Vec::new();
        for each in input {
            let parsed = Key::from_str(each.as_ref())?;
            ret.push(parsed);
        }

        Ok(ret)
    }

    fn from_str(s: &str) -> WmResult<Self> {
        // TODO: always match lower case
        let key = match s {
            "Esc" => Key::Esc,
            "F1" | "f1" => Key::F1,
            "F2" | "f2" => Key::F2,
            "F3" | "f3" => Key::F3,
            "F4" | "f4" => Key::F4,
            "F5" | "f5" => Key::F5,
            "F6" | "f6" => Key::F6,
            "F7" | "f7" => Key::F7,
            "F8" | "f8" => Key::F8,
            "F9" | "f9" => Key::F9,
            "F10" | "f10" => Key::F10,
            "F11" | "f11" => Key::F11,
            "F12" | "f12" => Key::F12,
            "Print" | "print" => Key::Print,
            "ScrollLock" | "scrolllock" => Key::ScrollLock,
            "Pause" | "pause" => Key::Pause,
            "`" => Key::Backtick,
            "1" => Key::Key1,
            "2" => Key::Key2,
            "3" => Key::Key3,
            "4" => Key::Key4,
            "5" => Key::Key5,
            "6" => Key::Key6,
            "7" => Key::Key7,
            "8" => Key::Key8,
            "9" => Key::Key9,
            "0" => Key::Key0,
            "-" => Key::Minus,
            "=" => Key::Equals,
            "Backspace" | "backpsace" => Key::Backspace,
            "Insert" | "insert" => Key::Insert,
            "Home" | "home" => Key::Home,
            "Prior" | "PgUp" | "PageUp" => Key::Prior,
            "NumLock" | "numlock" => Key::NumLock,
            "NumDivide" | "numdivide" => Key::NumDivide,
            "NumMultiply" | "nummultiply" => Key::NumMultiply,
            "NumSubstract" | "numsubstract" => Key::NumSubstract,
            "Tab" | "tab" => Key::Tab,
            "q" => Key::KeyQ,
            "w" => Key::KeyW,
            "e" => Key::KeyE,
            "r" => Key::KeyR,
            "t" => Key::KeyT,
            "y" => Key::KeyY,
            "u" => Key::KeyU,
            "i" => Key::KeyI,
            "o" => Key::KeyO,
            "p" => Key::KeyP,
            "]" => Key::RightAngleBracket,
            "[" => Key::LeftAngleBracket,
            "\\" => Key::Backslash,
            "Delete" | "delete" => Key::Delete,
            "End" | "end" => Key::End,
            "Next" | "PageDown" | "PgDown" => Key::Next,
            "Num7" | "num7" => Key::Num7,
            "Num8" | "num8" => Key::Num8,
            "Num9" | "num9" => Key::Num9,
            "NumAdd" | "numadd" => Key::NumAdd,
            "CapsLock" | "Caps" | "capslock" | "caps" => Key::CapsLock,
            "a" => Key::KeyA,
            "s" => Key::KeyS,
            "d" => Key::KeyD,
            "f" => Key::KeyF,
            "g" => Key::KeyG,
            "h" => Key::KeyH,
            "j" => Key::KeyJ,
            "k" => Key::KeyK,
            "l" => Key::KeyL,
            ";" => Key::Semicolon,
            "'" => Key::Quote,
            "CR" | "cr" | "Enter" | "enter" => Key::Enter,
            "Num4" | "num4" => Key::Num4,
            "Num5" | "num5" => Key::Num5,
            "Num6" | "num6" => Key::Num6,
            "Shift" | "LShift" | "lshift" | "shift" => Key::LShift,
            "z" => Key::KeyZ,
            "x" => Key::KeyX,
            "c" => Key::KeyC,
            "v" => Key::KeyV,
            "b" => Key::KeyB,
            "n" => Key::KeyN,
            "m" => Key::KeyM,
            "," => Key::Colon,
            "." => Key::Period,
            "/" => Key::Slash,
            "RShift" | "rshift" => Key::RShift,
            "up" | "Up" => Key::Up,
            "Num1" | "num1" => Key::Num1,
            "Num2" | "num2" => Key::Num2,
            "Num3" | "num3" => Key::Num3,
            "NumEnter" | "numenter" => Key::NumEnter,
            "Ctrl" | "ctrl" | "LCtrl" | "lctrl" | "C" => Key::Ctrl,
            "Mod" | "mod" => Key::Mod,
            "Alt" | "alt" | "LAlt" | "lalt" => Key::Alt,
            "Space" | "space" => Key::Space,
            "RAlt" | "ralt" => Key::RAlt,
            "Fn" | "fn" => Key::Fn,
            "Menu" | "menu" => Key::Menu,
            "RCtrl" | "rctrl" => Key::RCtrl,
            "Down" | "down" => Key::Down,
            "Left" | "left" => Key::Left,
            "Right" | "right" => Key::Right,
            "Num0" | "num0" => Key::Num0,
            "NumDecimal" | "numdecimal" => Key::NumDecimal,

            _ => return Err(format!("key parsing error: Unknown key {s}").into()),
        };
        Ok(key)
    }
}

#[derive(Debug)]
#[allow(unused)]
pub struct Keybind {
    keys: Vec<Key>,
    action: Action,
}

impl Keybind {
    fn new(keys: Vec<Key>, action: Action) -> Self {
        Self { keys, action }
    }

    fn from(str_keys: String, str_action: String) -> WmResult<Self> {
        let keys = Keybind::parse_keys(str_keys)?;
        let action = Keybind::parse_action(str_action)?;

        Ok(Self { keys, action })
    }

    fn parse_keys(input_keys: String) -> WmResult<Vec<Key>> {
        let mut ret = Vec::new();
        let mut special: Vec<String> = Vec::new();
        let mut is_special = false;
        let mut current_char;
        let mut keys = input_keys.chars().rev().collect::<String>();

        while !keys.is_empty() {
            current_char = keys.pop().unwrap();
            // if we are parsing a '<' block
            if current_char == '<' {
                if is_special {
                    return Err(format!("key parsing error: when parsing {input_keys}, invalid character {current_char}").into());
                } else {
                    is_special = true
                }
            } else if is_special {
                // if we are in a '<' block and the current character is not '-' or '>'
                if special.is_empty() && (current_char != '-' || current_char != '>') {
                    // push a new string to the special vector and add add the character in
                    special.push(String::from(current_char))
                } else if special.is_empty() && (current_char == '>') {
                    is_special = false;
                } else if !special.is_empty() {
                    if current_char == '-' {
                        special.push(String::new())
                    } else if current_char == '>' {
                        // parse the current vec of keys, return from special mode and clear
                        // vector of special keys
                        let mut parsed_special = Key::from_vec(&special)?;
                        ret.append(&mut parsed_special);
                        special.clear();
                        is_special = false
                    } else if let Some(last) = special.last_mut() {
                        last.push(current_char)
                    }
                }
            } else {
                let key = Key::from_str(&current_char.to_string())?;
                ret.push(key)
            }
        }
        Ok(ret)
    }

    fn parse_action(str_action: String) -> WmResult<Action> {
        Action::from_str(str_action)
    }
}

#[cfg(test)]
mod tests {
    use super::Key;
    use super::Keybind;

    #[test]
    fn test_keybind_parsing() {
        let keys = Keybind::parse_keys("<Mod-CR>".to_string());

        assert_eq!(keys.is_ok(), true);

        println!("{:#?}", keys.unwrap())
    }

    #[test]
    fn test_key_from_codes() {
        let codes: Vec<u8> = vec![0x01, 0x66, 0x22];

        let keys: Vec<Key> = codes
            .iter()
            .map(|&each| {
                let x = each.try_into();
                if x.is_ok() {
                    return x.unwrap();
                } else {
                    panic!("Failed to parse keycodes to keys.")
                }
            })
            .collect();

        assert_eq!(keys, vec![Key::Esc, Key::Mod, Key::NumLock]);
    }
}
