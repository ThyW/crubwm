use crate::{
    config::Repr,
    errors::WmResult,
    wm::actions::{Action, Direction},
};

use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct Keybinds(Vec<Keybind>);

impl Default for Keybinds {
    fn default() -> Self {
        let default_binds = vec![
            Keybind::new(
                vec![Key::Mod, Key::Enter],
                Action::Execute("xterm".to_string()),
            ),
            Keybind::new(vec![Key::Mod, Key::KeyK], Action::Kill),
            Keybind::new(vec![Key::Mod, Key::Key1], Action::Goto(1)),
            Keybind::new(vec![Key::Mod, Key::Key2], Action::Goto(2)),
            Keybind::new(vec![Key::Mod, Key::Key3], Action::Goto(3)),
            Keybind::new(vec![Key::Mod, Key::Key4], Action::Goto(4)),
            Keybind::new(vec![Key::Mod, Key::Key5], Action::Goto(5)),
            Keybind::new(vec![Key::Mod, Key::Key6], Action::Goto(6)),
            Keybind::new(vec![Key::Mod, Key::Key7], Action::Goto(7)),
            Keybind::new(vec![Key::Mod, Key::Key8], Action::Goto(8)),
            Keybind::new(vec![Key::Mod, Key::Key9], Action::Goto(9)),
            Keybind::new(vec![Key::Mod, Key::LShift, Key::Key1], Action::Move(1)),
            Keybind::new(vec![Key::Mod, Key::LShift, Key::Key2], Action::Move(2)),
            Keybind::new(vec![Key::Mod, Key::LShift, Key::Key3], Action::Move(3)),
            Keybind::new(vec![Key::Mod, Key::LShift, Key::Key4], Action::Move(4)),
            Keybind::new(vec![Key::Mod, Key::LShift, Key::Key5], Action::Move(5)),
            Keybind::new(vec![Key::Mod, Key::LShift, Key::Key6], Action::Move(6)),
            Keybind::new(vec![Key::Mod, Key::LShift, Key::Key7], Action::Move(7)),
            Keybind::new(vec![Key::Mod, Key::LShift, Key::Key8], Action::Move(8)),
            Keybind::new(vec![Key::Mod, Key::LShift, Key::Key9], Action::Move(9)),
            Keybind::new(vec![Key::Mod, Key::KeyL], Action::Focus(Direction::Next)),
            Keybind::new(
                vec![Key::Mod, Key::KeyH],
                Action::Focus(Direction::Previous),
            ),
            Keybind::new(
                vec![Key::Mod, Key::LShift, Key::KeyL],
                Action::Swap(Direction::Next),
            ),
            Keybind::new(
                vec![Key::Mod, Key::LShift, Key::KeyH],
                Action::Swap(Direction::Previous),
            ),
            Keybind::new(vec![Key::Mod, Key::KeyS], Action::CycleLayout),
            Keybind::new(vec![Key::Mod, Key::Space], Action::ToggleFloat),
            Keybind::new(vec![Key::Mod, Key::LShift, Key::KeyR], Action::ReloadConfig),
        ];
        Self(default_binds)
    }
}

impl Repr for Keybinds {
    fn repr(&self) -> WmResult<String> {
        let mut return_string = String::new();

        for keybind in self.0.iter() {
            return_string.push_str("keybind ");
            for (ii, key) in keybind.keys.iter().enumerate() {
                if ii == 0 {
                    return_string.push('"');
                }

                if key.is_special() {
                    write!(return_string, "<{}>", key.get_x11_str())?;
                } else {
                    return_string.push_str(key.get_x11_str())
                }
            }

            return_string.push('"');

            write!(return_string, " {}", keybind.action.repr()?)?;
            return_string.push('\n');
        }

        Ok(return_string)
    }
}

impl Keybinds {
    /// Add a new keybind.
    pub fn add(&mut self, keys: String, action: String) -> WmResult {
        let keybind = Keybind::from(keys, action)?;
        let mut remove_index = None;

        for (i, in_keybind) in self.0.iter().enumerate() {
            if in_keybind.action == keybind.action || in_keybind.keys == keybind.keys {
                remove_index = Some(i)
            }
        }

        if let Some(to_remove) = remove_index {
            self.0.remove(to_remove);
        }

        self.0.push(keybind);
        Ok(())
    }

    pub fn _extend(&mut self, from: Vec<Keybind>) {
        self.0.extend(from)
    }

    /// Get the X11 keysym names and action associated with the keybind.
    pub fn get_names_and_actions(&self) -> Vec<(Vec<&'_ str>, Action)> {
        let mut ret = Vec::with_capacity(self.0.len());
        for each in &self.0 {
            let names: Vec<&'_ str> = each.keys.iter().map(|k| k.get_x11_str()).collect();
            ret.push((names, each.action.clone()))
        }

        ret
    }

    /// Get only the X11 keysym names associated with a keybind.
    pub fn get_names(&self) -> Vec<Vec<&str>> {
        let mut ret = Vec::new();
        for each in &self.0 {
            let names: Vec<&'_ str> = each.keys.iter().map(|k| k.get_x11_str()).collect();
            ret.push(names)
        }

        ret
    }
}

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
    NumSubtract,
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

    fn is_special(&self) -> bool {
        if self.get_x11_str().len() > 1 {
            return true;
        }
        false
    }

    pub fn get_x11_str(&self) -> &'_ str {
        match self {
            Key::Esc => "Escape",
            Key::Key1 => "1",
            Key::Key2 => "2",
            Key::Key3 => "3",
            Key::Key4 => "4",
            Key::Key5 => "5",
            Key::Key6 => "6",
            Key::Key7 => "7",
            Key::Key8 => "8",
            Key::Key9 => "9",
            Key::Key0 => "0",
            Key::Minus => "minus",
            Key::Equals => "equal",
            Key::Backspace => "BackSpace",
            Key::Tab => "Tab",
            Key::KeyQ => "q",
            Key::KeyW => "w",
            Key::KeyE => "e",
            Key::KeyR => "r",
            Key::KeyT => "t",
            Key::KeyY => "y",
            Key::KeyU => "u",
            Key::KeyI => "i",
            Key::KeyO => "o",
            Key::KeyP => "p",
            Key::LeftAngleBracket => "bracketleft",
            Key::RightAngleBracket => "bracketright",
            Key::Enter => "Return",
            Key::Ctrl => "Control_L",
            Key::KeyA => "a",
            Key::KeyS => "s",
            Key::KeyD => "d",
            Key::KeyF => "f",
            Key::KeyG => "g",
            Key::KeyH => "h",
            Key::KeyJ => "j",
            Key::KeyK => "k",
            Key::KeyL => "l",
            Key::Semicolon => "semicolon",
            Key::Quote => "apostrophe",
            Key::Backtick => "grave",
            Key::LShift => "Shift_L",
            Key::Backslash => "backslash",
            Key::KeyZ => "z",
            Key::KeyX => "x",
            Key::KeyC => "c",
            Key::KeyV => "v",
            Key::KeyB => "b",
            Key::KeyN => "n",
            Key::KeyM => "m",
            Key::Colon => "comma",
            Key::Period => "period",
            Key::Slash => "slash",
            Key::RShift => "Shift_R",
            Key::NumMultiply => "KP_Multiply",
            Key::Alt => "Alt_L",
            Key::Space => "space",
            Key::CapsLock => "Caps_Lock",
            Key::F1 => "F1",
            Key::F2 => "F2",
            Key::F3 => "F3",
            Key::F4 => "F4",
            Key::F5 => "F5",
            Key::F6 => "F6",
            Key::F7 => "F7",
            Key::F8 => "F8",
            Key::F9 => "F9",
            Key::F10 => "F10",
            Key::NumLock => "Num_Lock",
            Key::ScrollLock => "Scroll_Lock",
            Key::NumSubtract => "KP_Subtract",
            Key::NumAdd => "KP_Add",
            Key::Num7 => "KP_Home",
            Key::Num8 => "KP_Up",
            Key::Num9 => "KP_Prior",
            Key::Num4 => "KP_Left",
            Key::Num5 => "KP_Begin",
            Key::Num6 => "KP_Right",
            Key::Num1 => "KP_End",
            Key::Num2 => "KP_Down",
            Key::Num3 => "KP_Next",
            Key::Num0 => "KP_Insert",
            Key::NumDecimal => "KP_Delete",
            Key::F11 => "F11",
            Key::F12 => "F12",
            Key::NumEnter => "KP_Enter",
            Key::RCtrl => "Control_R",
            Key::NumDivide => "KP_Divide",
            Key::Print => "Print",
            Key::RAlt => "Alt_R",
            Key::Home => "Home",
            Key::Up => "Up",
            Key::Prior => "Prior",
            Key::Left => "Left",
            Key::Right => "Right",
            Key::End => "End",
            Key::Down => "Down",
            Key::Next => "Next",
            Key::Insert => "Insert",
            Key::Delete => "Delete",
            Key::Pause => "Pause",
            Key::Mod => "Super_L",
            Key::Menu => "Menu",
            Key::Fn => "Fn key",
            Key::Noop => "Noop",
        }
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
            "pgup" | "pageup" | "prior" => Key::Prior,
            "numlock" => Key::NumLock,
            "numdivide" => Key::NumDivide,
            "nummultiply" => Key::NumMultiply,
            "numsubtract" => Key::NumSubtract,
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
            "up" => Key::Up,
            "num1" => Key::Num1,
            "num2" => Key::Num2,
            "num3" => Key::Num3,
            "numenter" => Key::NumEnter,
            "ctrl" | "control_l" | "lctrl" => Key::Ctrl,
            "super_l" | "mod" => Key::Mod,
            "alt_l" | "alt" => Key::Alt,
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

#[derive(Debug, Clone)]
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
    use crate::config::Repr;

    use super::Key;
    use super::Keybind;
    use super::Keybinds;

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

    #[test]
    fn test_repr() {
        let keybinds = Keybinds::default();

        println!("{}", keybinds.repr().unwrap())
    }
}
