mod keybinds;
pub mod keysyms;
mod options;
mod start_hooks;

pub use keybinds::*;
use options::*;
use start_hooks::*;

/// A representation of a parsed configuration file with all the options, hooks and keybinds for
/// the window manager.
#[derive(Debug)]
#[allow(unused)]
pub struct Config {
    pub keybinds: Keybinds,
    pub options: Options,
    pub start_hooks: StartHooks,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keybinds: Keybinds::default(),
            options: Options::default(),
            start_hooks: StartHooks::default(),
        }
    }
}
