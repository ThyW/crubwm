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

// TODO: This is all wrong and a better way to do this should be found.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[allow(unused)]
pub enum Key {
    Noop = 0x00,
    Esc = 0x09,
    // This key is commented out because I have not yet found a keycode for this key.
    // F1 = 0xff,
    F2 = 0x44,
    F3 = 0x45,
    F4 = 0x46,
    F5 = 0x47,
    F6 = 0x48,
    F7 = 0x49,
    F8 = 0x4a,
    F9 = 0x4b,
    F10 = 0x4c,
    F11 = 0x5f,
    F12 = 0x60,
    // Print = 0x0e,
    // ScrollLock = 0x0f,
    Pause = 0x6e,
    Backtick = 0x31,
    Key1 = 0x0a,
    Key2 = 0x0b,
    Key3 = 0x0c,
    Key4 = 0x0d,
    Key5 = 0x0e,
    Key6 = 0x0f,
    Key7 = 0x08,
    Key8 = 0x11,
    Key9 = 0x12,
    Key0 = 0x13,
    Minus = 0x14,
    Equals = 0x15,
    Backspace = 0x16,
    Insert = 0x6a,
    Home = 0x61,
    Prior = 0x63,
    // NumLock = 0x22,
    NumDivide = 0x70,
    NumMultiply = 0x3f,
    NumSubstract = 0x52,
    Tab = 0x17,
    KeyQ = 0x18,
    KeyW = 0x19,
    KeyE = 0x1a,
    KeyR = 0x1b,
    KeyT = 0x1c,
    KeyY = 0x1d,
    KeyU = 0x1e,
    KeyI = 0x1f,
    KeyO = 0x20,
    KeyP = 0x21,
    LeftAngleBracket = 0x22,
    RightAngleBracket = 0x23,
    Backslash = 0x33,
    Delete = 0x6b,
    End = 0x67,
    // Next = 0x36,
    // Num7 = 0x37,
    Num8 = 0x50,
    // Num9 = 0x39,
    NumAdd = 0x56,
    CapsLock = 0x42,
    KeyA = 0x26,
    KeyS = 0x27,
    KeyD = 0x28,
    KeyF = 0x29,
    KeyG = 0x2a,
    KeyH = 0x2b,
    KeyJ = 0x2c,
    KeyK = 0x2d,
    KeyL = 0x2e,
    Semicolon = 0x2f,
    Quote = 0x30,
    Enter = 0x24,
    Num4 = 0x53,
    Num5 = 0x54,
    Num6 = 0x55,
    LShift = 0x32,
    KeyZ = 0x34,
    KeyX = 0x35,
    KeyC = 0x36,
    KeyV = 0x37,
    KeyB = 0x38,
    KeyN = 0x39,
    KeyM = 0x3a,
    Colon = 0x3b,
    Period = 0x3c,
    Slash = 0x3d,
    RShift = 0x3e,
    Up = 0x62,
    Num1 = 0x57,
    Num2 = 0x58,
    // Num3 = 0x5a,
    NumEnter = 0x6c,
    Ctrl = 0x25,
    Mod = 0x73,
    Alt = 0x40,
    Space = 0x41,
    RAlt = 0x71,
    Fn = 0x74,
    Menu = 0x75,
    RCtrl = 0x6d,
    Down = 0x64,
    Left = 0x68,
    Right = 0x66,
    Num0 = 0x5a,
    NumDecimal = 0x5b,
}

impl TryFrom<u8> for Key {
    type Error = crate::errors::Error;

    fn try_from(c: u8) -> Result<Self, Self::Error> {
        println!("converting: {:x}", c);
        if c > 0x75 {
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
            // "F1" | "f1" => Key::F1,
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
            // "Print" | "print" => Key::Print,
            // "ScrollLock" | "scrolllock" => Key::ScrollLock,
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
            // "NumLock" | "numlock" => Key::NumLock,
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
            // "Next" | "PageDown" | "PgDown" => Key::Next,
            // "Num7" | "num7" => Key::Num7,
            "Num8" | "num8" => Key::Num8,
            // "Num9" | "num9" => Key::Num9,
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
            // "Num3" | "num3" => Key::Num3,
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
        let codes: Vec<u8> = vec![0x01, 0x66];

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

        assert_eq!(keys, vec![Key::Esc, Key::Mod]);
    }
}
