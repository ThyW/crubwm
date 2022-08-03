use crate::errors::WmResult;

use std::collections::HashMap;
use std::rc::Rc;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::AtomEnum;
use x11rb::protocol::xproto::ConnectionExt;

const MAX_VAL_NUMBER: u32 = 1024 * 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Single(AtomEnum),
    List(AtomEnum, usize),
    ListOfLists(usize, AtomEnum, usize),
}

#[derive(Clone, Debug)]
pub enum PropertyReturnValue {
    String(String),
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

    // TODO this should be fixed a bit
    pub fn value_number(&self) -> u32 {
        match self.value_type() {
            ValueType::Single(_) => MAX_VAL_NUMBER,
            ValueType::List(_, num) => {
                if num > 0 {
                    num as u32
                } else {
                    MAX_VAL_NUMBER
                }
            }
            ValueType::ListOfLists(..) => MAX_VAL_NUMBER * MAX_VAL_NUMBER,
        }
    }

    pub fn get_property(
        &self,
        window: u32,
        connection: Rc<impl Connection>,
    ) -> WmResult<Vec<PropertyReturnValue>> {
        let mut ret = Vec::new();

        let type_ = match self.value_type() {
            ValueType::Single(atom) => atom,
            ValueType::List(atom, _) => atom,
            ValueType::ListOfLists(_, atom, _) => atom,
        };

        let reply = connection
            .get_property(false, window, self.id(), type_, 0, self.value_number())?
            .reply()?;

        if type_ == AtomEnum::STRING {
            if let Some(value) = reply.value8() {
                ret.push(PropertyReturnValue::String(String::from_utf8(
                    value.collect::<Vec<u8>>(),
                )?));
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
            ("_NET_SUPPORTED", ValueType::List(AtomEnum::CARDINAL, 0)),
            ("_NET_CLIENT_LIST", ValueType::List(AtomEnum::WINDOW, 0)),
            (
                "_NET_NUMBER_OF_DESKTOPS",
                ValueType::List(AtomEnum::CARDINAL, 0),
            ),
            (
                "_NET_DESKTOP_GEOMETRY",
                ValueType::List(AtomEnum::CARDINAL, 2),
            ),
            (
                "_NET_DESKTOP_VIEWPORT",
                ValueType::ListOfLists(0, AtomEnum::CARDINAL, 2),
            ),
            (
                "_NET_CURRENT_DESKTOP",
                ValueType::Single(AtomEnum::CARDINAL),
            ),
            ("_NET_DESKTOP_NAMES", ValueType::List(AtomEnum::STRING, 0)),
            ("_NET_ACTIVE_WINDOW", ValueType::Single(AtomEnum::WINDOW)),
            (
                "_NET_WORKAREA",
                ValueType::ListOfLists(0, AtomEnum::CARDINAL, 4),
            ),
            (
                "_NET_SUPPORTING_WM_CHECK",
                ValueType::Single(AtomEnum::WINDOW),
            ),
            ("_NET_VIRTUAL_ROOTS", ValueType::List(AtomEnum::WINDOW, 0)),
            (
                "_NET_DESKTOP_LAYOUT",
                ValueType::List(AtomEnum::CARDINAL, 4),
            ),
            (
                "_NET_SHOWING_DESKTOP",
                ValueType::Single(AtomEnum::CARDINAL),
            ),
            // client messages
            ("_NET_WM_STATE", ValueType::List(AtomEnum::ATOM, 0)),
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
            ("_NET_WM_WINDOW_TYPE", ValueType::List(AtomEnum::ATOM, 0)),
            (
                "_NET_WM_ALLOWED_ACTIONS",
                ValueType::List(AtomEnum::ATOM, 0),
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
                ValueType::ListOfLists(0, AtomEnum::CARDINAL, 0),
            ),
            ("_NET_WM_PID", ValueType::Single(AtomEnum::CARDINAL)),
            // "_NET_WM_HANDLED_ICONS",
            ("_NET_WM_USER_TIME", ValueType::Single(AtomEnum::CARDINAL)),
            ("_NET_FRAME_EXTENTS", ValueType::List(AtomEnum::CARDINAL, 4)),
            ("WM_NAME", ValueType::Single(AtomEnum::STRING)),
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
