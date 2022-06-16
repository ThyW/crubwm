pub mod keybinds;
pub mod keysyms;
pub mod options;
pub mod start_hooks;
pub mod workspace_settings;

pub use crate::errors::WmResult;
pub use keybinds::*;
pub use options::*;
pub use start_hooks::*;
pub use workspace_settings::*;

/// A representation of a parsed configuration file with all the options, hooks and keybinds for
/// the window manager.
#[derive(Debug, Default, Clone)]
#[allow(unused)]
pub struct Config {
    pub keybinds: Keybinds,
    pub options: Options,
    pub start_hooks: StartHooks,
    pub workspace_settings: AllWorkspaceSettings,
    pub path: String,
}

impl Config {
    pub fn serialize(&self) -> WmResult<&[u8]> {
        let mut string = String::new();

        string.push_str(&self.keybinds.repr()?);
        string.push_str(&self.options.repr()?);

        Ok(&[])
    }
}

pub trait Repr {
    fn repr(&self) -> WmResult<String>;
}
