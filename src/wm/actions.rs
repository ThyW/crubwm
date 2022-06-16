use crate::{
    config::Repr,
    errors::{Error, WmResult},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Next,
    Previous,
}

impl Repr for Direction {
    fn repr(&self) -> WmResult<String> {
        match &self {
            Self::Next => Ok("next".to_string()),
            Self::Previous => Ok("previous".to_string()),
        }
    }
}

impl TryFrom<&str> for Direction {
    type Error = Error;

    fn try_from(s: &str) -> WmResult<Self> {
        let direction = match s {
            "next" => Self::Next,
            "previous" => Self::Previous,
            _ => return Err("not a valid direction".into()),
        };

        Ok(direction)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// A noop, doesn't do anything.
    Noop,
    /// Run a system command.
    Execute(String),
    /// Kill currently focused window.
    Kill,
    /// Switch focus to a workspace, given its ID.
    Goto(usize),
    /// Move currently focused window to a given workspace ID.
    Move(usize),
    /// Shift foucs in a direction.
    ///
    /// Some layouts use `Up` and `Down` as directions, others use `Left` and `Right`.
    Focus(Direction),
    /// Apply a different layout to the currently focused workspace.
    ChangeLayout(String),
    /// Cycle layouts for the currently focused workspace.
    CycleLayout,
    /// Toggle the currently focused window in and out of floating.
    ToggleFloat,
    /// Swap two clients.
    Swap(Direction),
    /// Reload a configuration file
    ReloadConfig,
}

impl Action {
    /// Attetmpt to parse a string into an `Action`.
    ///
    /// More about this can be found in the `config` and `parsers` section of the documentation.
    pub fn from_str(s: String) -> WmResult<Self> {
        if s.is_empty() {
            Err("action paring error: Action is empty!".into())
        } else {
            let parts = s.split(' ').collect::<Vec<&str>>();
            let action = match parts[0] {
                "noop" => Action::Noop,
                "execute" | "exec" => {
                    let rest = &parts[1..];
                    let mut buff = String::new();

                    for each in rest {
                        buff.push_str(each)
                    }

                    Action::Execute(buff)
                }
                "kill" => Action::Kill,
                "goto" => {
                    let rest = &parts[1..];
                    if rest.len() > 1 {
                        return Err(format!(
                            "action parsing error: Action takes exactly one argument {s}"
                        )
                        .into());
                    } else {
                        let num = rest[0].parse::<usize>();
                        if let Ok(n) = num {
                            Action::Goto(n)
                        } else {
                            return Err(format!(
                                "action paring error: Argument must be a number {s}"
                            )
                            .into());
                        }
                    }
                }
                "move" => {
                    let rest = &parts[1..];
                    if rest.len() > 1 {
                        return Err(format!(
                            "action parsing error: Action takes exactly one argument {s}"
                        )
                        .into());
                    } else {
                        let num = rest[0].parse::<usize>();
                        if let Ok(n) = num {
                            Action::Move(n)
                        } else {
                            return Err(format!(
                                "action paring error: Argument must be a number {s}"
                            )
                            .into());
                        }
                    }
                }
                "focus" => {
                    let rest = &parts[1..];
                    if rest.len() > 1 {
                        return Err(format!(
                            "action parsing error: Action takes exactly one argument {s}"
                        )
                        .into());
                    } else {
                        let direction = rest[0].try_into();
                        if let Ok(dir) = direction {
                            Action::Focus(dir)
                        } else {
                            return Err(format!(
                                "action paring error: Argument must be a number {s}"
                            )
                            .into());
                        }
                    }
                }
                "change_layout" => {
                    let rest = &parts[1..];
                    if rest.is_empty() {
                        return Err(format!("action parsing error: Action takes one argument, but zero were supplied {s}").into());
                    } else if rest.len() > 1 {
                        return Err(format!(
                            "action parsing error: Action takes exactly one argument {s}"
                        )
                        .into());
                    } else {
                        let layout = rest[0];
                        return Ok(Action::ChangeLayout(layout.into()));
                    }
                }
                "cycle_layout" => Action::CycleLayout,
                "toggle_float" => Action::ToggleFloat,
                "swap" => {
                    let rest = &parts[1..];
                    if rest.len() > 1 {
                        return Err(format!(
                            "action parsing error: Action takes exactly one argument {s}"
                        )
                        .into());
                    } else {
                        let direction = rest[0].try_into();
                        if let Ok(dir) = direction {
                            Action::Swap(dir)
                        } else {
                            return Err(format!(
                                "action paring error: Argument must be a number {s}"
                            )
                            .into());
                        }
                    }
                }
                "reload_config" => Action::ReloadConfig,
                a => return Err(format!("action parsing error: Unknown action {a}!").into()),
            };

            Ok(action)
        }
    }
}

impl Repr for Action {
    fn repr(&self) -> WmResult<String> {
        match self {
            &Self::Goto(workspace) => Ok(format!("goto {workspace}")),
            &Self::Noop => Ok("noop".to_string()),
            &Self::Kill => Ok("kill".to_string()),
            Self::Execute(command) => Ok(format!("execute {command}")),
            &Self::Move(workspace) => Ok(format!("move {workspace}")),
            &Self::Focus(direction) => Ok(format!("focus {}", direction.repr()?)),
            &Self::ToggleFloat => Ok("toggle_float".to_string()),
            &Self::CycleLayout => Ok("cycle_layout".to_string()),
            Self::ChangeLayout(name) => Ok(format!("change_layout {name}")),
            &Self::Swap(direction) => Ok(format!("swap {}", direction.repr()?)),
            &Self::ReloadConfig => Ok("reload_config".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn direction_repr() {
        let dir = super::Direction::Next;
        let str = dir.repr();

        assert_eq!(str.unwrap(), "next".to_string());
    }
}
