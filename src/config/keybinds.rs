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

// TODO: Keys have to be able to be turned into a X-Keysym-name compatible strings, in order to be
// able to grab keys.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[allow(unused)]
pub enum Key {
    ScrollLock,
    Noop,
    Esc,
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
    Print,
    SrollLock,
    Pause,
    Backtick,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key0,
    Minus,
    Equals,
    Backspace,
    Insert,
    Home,
    Prior,
    NumLock,
    NumDivide,
    NumMultiply,
    NumSubstract,
    Tab,
    KeyQ,
    KeyW,
    KeyE,
    KeyR,
    KeyT,
    KeyY,
    KeyU,
    KeyI,
    KeyO,
    KeyP,
    LeftAngleBracket,
    RightAngleBracket,
    Backslash,
    Delete,
    End,
    Next,
    Num7,
    Num8,
    Num9,
    NumAdd,
    CapsLock,
    KeyA,
    KeyS,
    KeyD,
    KeyF,
    KeyG,
    KeyH,
    KeyJ,
    KeyK,
    KeyL,
    Semicolon,
    Quote,
    Enter,
    Num4,
    Num5,
    Num6,
    LShift,
    KeyZ,
    KeyX,
    KeyC,
    KeyV,
    KeyB,
    KeyN,
    KeyM,
    Colon,
    Period,
    Slash,
    RShift,
    Up,
    Num1,
    Num2,
    Num3,
    NumEnter,
    Ctrl,
    Mod,
    Alt,
    Space,
    RAlt,
    Fn,
    Menu,
    RCtrl,
    Down,
    Left,
    Right,
    Num0,
    NumDecimal,
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

    pub fn from_str(s: &str) -> WmResult<Self> {
        let key = match s.to_lowercase().as_str() {
            "esc" | "escape" => Key::Esc,
            "f1" => Key::F1,
            "f2" => Key::F2,
            "f3" => Key::F3,
            "f4" => Key::F4,
            "f5" => Key::F5,
            "f6" => Key::F6,
            "f7" => Key::F7,
            "f8" => Key::F8,
            "f9" => Key::F9,
            "f10" => Key::F10,
            "f11" => Key::F11,
            "f12" => Key::F12,
            "print" => Key::Print,
            "scroll_lock" => Key::ScrollLock,
            "pause" => Key::Pause,
            "`" | "backtick" | "grave" => Key::Backtick,
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
            "-" | "minus" => Key::Minus,
            "=" | "equal" => Key::Equals,
            "backspace" => Key::Backspace,
            "insert" => Key::Insert,
            "home" => Key::Home,
            "pgup" | "pageup" | "prior"=> Key::Prior,
            "numlock" => Key::NumLock,
            "numdivide" => Key::NumDivide,
            "nummultiply" => Key::NumMultiply,
            "numsubstract" => Key::NumSubstract,
            "tab" => Key::Tab,
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
            "]" | "bracketright" => Key::RightAngleBracket,
            "[" | "bracketleft" => Key::LeftAngleBracket,
            "\\" | "backslash" => Key::Backslash,
            "Delete" | "delete" => Key::Delete,
            "End" | "end" => Key::End,
            "next" | "pagedown" | "pgdown" => Key::Next,
            "num7" => Key::Num7,
            "num8" => Key::Num8,
            "num9" => Key::Num9,
            "numadd" => Key::NumAdd,
            "caps_lock" | "caps" => Key::CapsLock,
            "a" => Key::KeyA,
            "s" => Key::KeyS,
            "d" => Key::KeyD,
            "f" => Key::KeyF,
            "g" => Key::KeyG,
            "h" => Key::KeyH,
            "j" => Key::KeyJ,
            "k" => Key::KeyK,
            "l" => Key::KeyL,
            ";" | "semicolon" => Key::Semicolon,
            "'" | "apostrophe" => Key::Quote,
            "cr" | "enter" | "return" => Key::Enter,
            "num4" => Key::Num4,
            "num5" => Key::Num5,
            "num6" => Key::Num6,
            "shift_l" | "lshift" | "shift" => Key::LShift,
            "z" => Key::KeyZ,
            "x" => Key::KeyX,
            "c" => Key::KeyC,
            "v" => Key::KeyV,
            "b" => Key::KeyB,
            "n" => Key::KeyN,
            "m" => Key::KeyM,
            "," | "comma" => Key::Colon,
            "." | "period" => Key::Period,
            "/" | "slash" => Key::Slash,
            "shift_r" | "rshift" => Key::RShift,
            "up"=> Key::Up,
            "num1" => Key::Num1,
            "num2" => Key::Num2,
            "num3" => Key::Num3,
            "numenter" => Key::NumEnter,
            "ctrl" | "control_l" | "lctrl" | "C" => Key::Ctrl,
            "super_l" | "mod" => Key::Mod,
            "alt_l" |"alt" => Key::Alt,
            "space" => Key::Space,
            "alt_r" | "ralt" => Key::RAlt,
            "fn" => Key::Fn,
            "menu" => Key::Menu,
            "control_r" | "rctrl" => Key::RCtrl,
            "down" => Key::Down,
            "left" => Key::Left,
            "right" => Key::Right,
            "num0" => Key::Num0,
            "numdecimal" => Key::NumDecimal,

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
        use crate::config::keysyms::Keysym;
        use x11::xlib::XOpenDisplay;

        let dpy = unsafe { XOpenDisplay(std::ptr::null()) };

        let keysym = Keysym::keysym_from_keycode(dpy, 0x11, 0).unwrap();
        let key = Key::from_str(keysym.name().as_str()).unwrap();

        assert_eq!(Key::Key8, key);
    }
}
