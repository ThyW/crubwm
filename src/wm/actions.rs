use crate::errors::{Error, WmResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl TryFrom<&str> for Direction {
    type Error = Error;

    fn try_from(s: &str) -> WmResult<Self> {
        let direction = match s {
            "up" => Self::Up,
            "down" => Self::Down,
            "left" => Self::Left,
            "right" => Self::Right,
            _ => return Err("not a valid direction".into()),
        };

        Ok(direction)
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    /// A noop, doesn't do anything.
    Noop,
    /// Run a system command.
    Execute(String),
    /// Kill currently confused window.
    Kill,
    /// Switch focus to a workspace, given its ID.
    Goto(usize),
    /// Move currently focused window to a given workspace ID.
    Move(usize),
    /// Shift foucs in a direction
    Focus(Direction),
    /// Apply a different layout to the currently focused workspace
    ChangeLayout(String),
    /// Cycle layouts for the currently focused workspace
    CycleLayout,
}

impl Action {
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
                a => return Err(format!("action parsing error: Unknown action {a}!").into()),
            };

            Ok(action)
        }
    }
}
