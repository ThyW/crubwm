use crate::errors::WmResult;

use std::collections::HashMap;
use std::sync::Arc;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::AtomEnum;
use x11rb::protocol::xproto::ClientMessageEvent;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::protocol::xproto::EventMask;

const FOURKB: usize = 1024 * 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Single(AtomEnum),
    List(AtomEnum, usize),
    ListOfLists(usize, AtomEnum, usize),
}

impl ValueType {
    #[allow(unused)]
    pub fn atom(&self) -> &AtomEnum {
        match self {
            Self::Single(a) => a,
            Self::List(a, _) => a,
            Self::ListOfLists(_, a, _) => a,
        }
    }
}

#[derive(Clone, Debug)]
pub enum PropertyReturnValue {
    String(String),
    Byte(u8),
    DoubleByte(u16),
    Number(u32),
}

impl TryInto<u32> for PropertyReturnValue {
    type Error = crate::errors::Error;
    fn try_into(self) -> Result<u32, Self::Error> {
        if let Self::Number(number) = self {
            return Ok(number);
        }
        Err("Unable to return the correct Atom type".into())
    }
}

impl TryInto<String> for PropertyReturnValue {
    type Error = crate::errors::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        if let Self::String(string) = self {
            return Ok(string);
        }
        Err("Unable to return the correct Atom type".into())
    }
}

pub struct AtomManager;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AtomStruct {
    name: &'static str,
    x_id: u32,
    value: ValueType,
}

impl AtomStruct {
    pub fn new(name: &'static str, x_id: impl Into<u32>, value: ValueType) -> Self {
        Self {
            name,
            x_id: x_id.into(),
            value,
        }
    }

    pub fn _name(&self) -> &str {
        self.name
    }

    pub fn id(&self) -> u32 {
        self.x_id
    }

    pub fn value_type(&self) -> ValueType {
        self.value
    }

    pub fn byte_amount(&self, format: Option<u8>) -> usize {
        let format = format.unwrap_or(32) as usize;
        match self.value_type() {
            ValueType::Single(_) => FOURKB as usize,
            ValueType::List(_, len) => len * format,
            ValueType::ListOfLists(len1, _, len2) => len1 * len2 * format,
        }
    }

    pub fn get_property(
        &self,
        window: u32,
        connection: Arc<impl Connection>,
        format: Option<u8>,
    ) -> WmResult<Vec<PropertyReturnValue>> {
        let mut ret = Vec::new();

        let type_ = match self.value_type() {
            ValueType::Single(atom) => atom,
            ValueType::List(atom, _) => atom,
            ValueType::ListOfLists(_, atom, _) => atom,
        };

        let reply = connection
            .get_property(
                false,
                window,
                self.id(),
                type_,
                0,
                self.byte_amount(format) as u32,
            )?
            .reply()?;

        if type_ == AtomEnum::STRING {
            if let Some(value) = reply.value8() {
                ret.push(PropertyReturnValue::String(String::from_utf8(
                    value.collect::<Vec<u8>>(),
                )?));
            }
        } else if let Some(fmt) = format {
            match fmt {
                8 => {
                    if let Some(value) = reply.value8() {
                        for each in value {
                            ret.push(PropertyReturnValue::Byte(each))
                        }
                    }
                }
                16 => {
                    if let Some(value) = reply.value16() {
                        for each in value {
                            ret.push(PropertyReturnValue::DoubleByte(each))
                        }
                    }
                }
                32 => {
                    if let Some(value) = reply.value32() {
                        for each in value {
                            ret.push(PropertyReturnValue::Number(each))
                        }
                    }
                }
                _ => return Err(format!("Invalid format: {fmt}").into()),
            }
        } else if let Some(value) = reply.value32() {
            for each in value {
                ret.push(PropertyReturnValue::Number(each))
            }
        }

        Ok(ret)
    }
}

impl AtomManager {
    /// Initialize all atoms.
    pub fn init_atoms(c: &impl Connection) -> WmResult<HashMap<String, AtomStruct>> {
        let mut hm = HashMap::new();
        // https://en.wikipedia.org/wiki/Extended_Window_Manager_Hints
        let atoms = [
            // root window
            (
                "_NET_SUPPORTED",
                ValueType::List(AtomEnum::CARDINAL, FOURKB),
            ),
            (
                "_NET_CLIENT_LIST",
                ValueType::List(AtomEnum::WINDOW, FOURKB),
            ),
            (
                "_NET_NUMBER_OF_DESKTOPS",
                ValueType::List(AtomEnum::CARDINAL, FOURKB),
            ),
            (
                "_NET_DESKTOP_GEOMETRY",
                ValueType::List(AtomEnum::CARDINAL, 2),
            ),
            (
                "_NET_DESKTOP_VIEWPORT",
                ValueType::ListOfLists(FOURKB, AtomEnum::CARDINAL, 2),
            ),
            (
                "_NET_CURRENT_DESKTOP",
                ValueType::Single(AtomEnum::CARDINAL),
            ),
            (
                "_NET_DESKTOP_NAMES",
                ValueType::List(AtomEnum::STRING, FOURKB),
            ),
            ("_NET_ACTIVE_WINDOW", ValueType::Single(AtomEnum::WINDOW)),
            (
                "_NET_WORKAREA",
                ValueType::ListOfLists(FOURKB, AtomEnum::CARDINAL, 4),
            ),
            (
                "_NET_SUPPORTING_WM_CHECK",
                ValueType::Single(AtomEnum::WINDOW),
            ),
            (
                "_NET_VIRTUAL_ROOTS",
                ValueType::List(AtomEnum::WINDOW, FOURKB),
            ),
            (
                "_NET_DESKTOP_LAYOUT",
                ValueType::List(AtomEnum::CARDINAL, 4),
            ),
            (
                "_NET_SHOWING_DESKTOP",
                ValueType::Single(AtomEnum::CARDINAL),
            ),
            // client messages
            ("_NET_WM_STATE", ValueType::List(AtomEnum::ATOM, FOURKB)),
            // "_NET_CLOSE_WINDOW",
            // "_NET_WM_MOVERESIZE",
            // "_NET_MOVERESIZE_WINDOW",
            // "_NET_REQUEST_FRAME_EXTENTS",
            // "_NET_WM_FULLSCREEN_MONITORS",
            // "_NET_RESTACK_WINDOW",
            // window properties
            ("_NET_WM_NAME", ValueType::Single(AtomEnum::STRING)),
            ("_NET_WM_VISIBLE_NAME", ValueType::Single(AtomEnum::STRING)),
            ("_NET_WM_ICON_NAME", ValueType::Single(AtomEnum::STRING)),
            (
                "_NET_WM_VISIBLE_ICON_NAME",
                ValueType::Single(AtomEnum::STRING),
            ),
            ("_NET_WM_DESKTOP", ValueType::Single(AtomEnum::CARDINAL)),
            (
                "_NET_WM_WINDOW_TYPE",
                ValueType::List(AtomEnum::ATOM, FOURKB),
            ),
            (
                "_NET_WM_ALLOWED_ACTIONS",
                ValueType::List(AtomEnum::ATOM, FOURKB),
            ),
            ("_NET_WM_STRUT", ValueType::List(AtomEnum::CARDINAL, 4)),
            (
                "_NET_WM_STRUT_PARTIAL",
                ValueType::List(AtomEnum::CARDINAL, 12),
            ),
            (
                "_NET_WM_ICON_GEOMETRY",
                ValueType::List(AtomEnum::CARDINAL, 4),
            ),
            (
                "_NET_WM_ICON",
                ValueType::ListOfLists(FOURKB, AtomEnum::CARDINAL, FOURKB),
            ),
            ("_NET_WM_PID", ValueType::Single(AtomEnum::CARDINAL)),
            // "_NET_WM_HANDLED_ICONS",
            ("_NET_WM_USER_TIME", ValueType::Single(AtomEnum::CARDINAL)),
            ("_NET_FRAME_EXTENTS", ValueType::List(AtomEnum::CARDINAL, 4)),
            ("WM_NAME", ValueType::Single(AtomEnum::STRING)),
            ("WM_DELETE_WINDOW", ValueType::Single(AtomEnum::ATOM)),
            ("WM_PROTOCOLS", ValueType::List(AtomEnum::ATOM, FOURKB)),
        ];

        for (atom, value) in atoms {
            let atom_value = c.intern_atom(false, atom.as_bytes())?.reply()?.atom;
            let atom_struct = AtomStruct::new(atom, atom_value, value);
            if atom_value == 0 {
                return Err(format!(
                    "x11 atom error: intern atom failed return ATOM_NONE for atom {atom}."
                )
                .into());
            }

            hm.insert(atom.into(), atom_struct);
        }

        Ok(hm)
    }
}

pub fn send_client_message(
    connection: Arc<impl Connection>,
    window: u32,
    atom: u32,
    format: u8,
    data: &[u8],
) -> WmResult {
    match format {
        8 => {
            let mut data = data.iter().take(20).copied().collect::<Vec<u8>>();

            while data.len() < 20 {
                data.push(0)
            }

            connection.send_event(
                true,
                window,
                EventMask::NO_EVENT,
                ClientMessageEvent::new(
                    format,
                    window,
                    atom,
                    <Vec<u8> as TryInto<[u8; 20]>>::try_into(data).unwrap(),
                ),
            )?;
        }
        16 => {
            let mut data: Vec<u16> = data
                .chunks_exact(2)
                .take(10)
                .map(|c| {
                    let mut _x = 0u16;
                    _x = (c[0] as u16) << 8 | c[1] as u16;
                    _x
                })
                .collect();

            while data.len() < 10 {
                data.push(0)
            }

            connection.send_event(
                true,
                window,
                EventMask::NO_EVENT,
                ClientMessageEvent::new(
                    format,
                    window,
                    atom,
                    <Vec<u16> as TryInto<[u16; 10]>>::try_into(data).unwrap(),
                ),
            )?;
        }
        32 => {
            let mut data: Vec<u32> = data
                .chunks(4)
                .take(5)
                .map(|c| {
                    let mut _x = 0u32;
                    _x = (c[0] as u32) << 24
                        | (c[1] as u32) << 16
                        | (c[2] as u32) << 8
                        | c[3] as u32;
                    _x
                })
                .collect();

            while data.len() < 5 {
                data.push(0)
            }

            connection.send_event(
                true,
                window,
                EventMask::NO_EVENT,
                ClientMessageEvent::new(
                    format,
                    window,
                    atom,
                    <Vec<u32> as TryInto<[u32; 5]>>::try_into(data).unwrap(),
                ),
            )?;
        }
        _ => return Err(format!("Invalid format for sending client message: {format}").into()),
    };
    Ok(())
}
