#![allow(unused)]

use crate::errors::Error;

use super::WmResult;

const POSITIONS: [&str; 3] = ["left", "right", "middle"];

#[derive(Clone, Debug)]
/// Settings for a single widget.
pub struct WidgetSettings {
    /// A name or an indetifier for the widget, chosen by the user.
    pub id: String,
    /// How the widget should be displayed.
    /// Can be a string of characters for example, 'BATTERY' or '', or it can be a string that
    /// starts with 'file:' followed by a path to an image wich will be attempted to be loaded and
    /// used as an icon.
    pub icon: String,
    /// A command that is run on every update.
    pub command: String,
    /// Time, in seconds, of how often should the widget be updated.
    pub update_time: u32,
    /// Font of the widget.
    pub font: String,
    /// Position of the segment
    pub position: String,
}

impl Default for WidgetSettings {
    fn default() -> Self {
        Self {
            id: "none".into(),
            icon: "NONE".into(),
            command: "".into(),
            update_time: 0,
            font: "monospace".into(),
            position: "right".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct WorkspaceSegmentSettings {
    /// Text color of the currently focused workspace.
    pub focused_foreground_color: String,
    /// Background color of the currently focused workspace.
    pub focused_background_color: String,
    /// Text color of the currently unfocused workspace.
    pub normal_foreground_color: String,
    /// Background color of the currently unfocused workspace.
    pub normal_background_color: String,
    /// Font used to display workspace segement text(Workspace name and id).
    pub font: String,
    /// Position of the segment
    pub position: String,
}

impl Default for WorkspaceSegmentSettings {
    fn default() -> Self {
        Self {
            focused_foreground_color: "#ffffff".to_string(),
            focused_background_color: "#00a2ff".to_string(),
            normal_foreground_color: "#ffffff".to_string(),
            normal_background_color: "#333333".to_string(),
            font: "monospace".to_string(),
            position: "left".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct WindowTitleSettings {
    /// Font used for displaying window title.
    pub font: String,
    /// Text color for the window title.
    pub foreground_color: String,
    /// Background color for the window title.
    pub background_color: String,
    /// Position of the segment.
    pub position: String,
}

impl Default for WindowTitleSettings {
    fn default() -> Self {
        Self {
            font: "monospace".into(),
            foreground_color: "#ffffff".into(),
            background_color: "00a2ff".into(),
            position: "middle".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct IconTraySettings {
    /// Position of the segment.
    pub position: String,
}

impl Default for IconTraySettings {
    fn default() -> Self {
        Self {
            position: "right".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BarSettings {
    /// identifier for the bar, unique 32-bit integer
    pub identifier: u32,
    /// monitor id of the bar
    pub monitor: u32,
    /// All the widget settings for the given monitor.
    pub widget_segment: Option<Vec<WidgetSettings>>,
    /// Workspace settings for the given monitor.
    pub workspace_segment: Option<WorkspaceSegmentSettings>,
    /// Window title settings for the given monitor.
    pub title_segment: Option<WindowTitleSettings>,
    // TODO: maybe have some settings
    /// Icon trya segment.
    pub icon_tray: Option<IconTraySettings>,
}

impl BarSettings {
    fn new(identifier: u32) -> Self {
        Self {
            identifier,
            monitor: 1,
            widget_segment: None,
            workspace_segment: None,
            title_segment: None,
            icon_tray: None,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct AllBarSettings(Vec<BarSettings>);

impl AllBarSettings {
    pub fn add(
        &mut self,
        bar_identifier: u32,
        bar_setting_name: String,
        bar_setting_values: Vec<String>,
    ) -> WmResult {
        let bar = match self.0.iter_mut().find(|bs| bs.identifier == bar_identifier) {
            Some(bar) => bar,
            None => {
                self.0.push(BarSettings::new(bar_identifier));
                self.0.last_mut().unwrap()
            }
        };

        match &bar_setting_name[..] {
            "monitor" => {
                bar.monitor = bar_setting_values[0].parse::<u32>()?;
            }
            "widget" => {
                if &bar_setting_values[0] == "add" {
                    // bar_set 0 widget add "battery" icon "" command "acpi" font "Iosevka" update_time "5"
                    let widget_segment = match &mut bar.widget_segment {
                        Some(ws) => ws,
                        None => {
                            bar.widget_segment = Some(Vec::new());
                            bar.widget_segment.as_mut().unwrap()
                        }
                    };
                    let name = bar_setting_values[1].clone();
                    let mut widget = WidgetSettings::default();
                    for (mut ii, value) in bar_setting_values[1..].iter().enumerate() {
                        ii += 1;
                        match &value[..] {
                            "icon" => {
                                widget.icon = bar_setting_values
                                    .get(ii + 1)
                                    .ok_or_else(|| {
                                        Error::Generic(format!("missing value for {value}"))
                                    })?
                                    .to_string();
                            }
                            "command" => {
                                let mut command_parts = Vec::new();
                                for command_segment in bar_setting_values[ii..].iter() {
                                    if command_segment == "icon"
                                        || command_segment == "update_time"
                                        || command_segment == "font"
                                    {
                                        break;
                                    } else {
                                        command_parts.push(command_segment.clone())
                                    }
                                }
                                if command_parts.is_empty() {
                                    return Err(format!(
                                        "the command field for widget named {} is empty.",
                                        widget.id
                                    )
                                    .into());
                                } else {
                                    widget.command = command_parts.join(" ");
                                    widget.command = widget.command.trim_end().to_string();
                                }
                            }
                            "update_time" => {
                                let digit = bar_setting_values.get(ii + 1).ok_or_else(|| {
                                    Error::Generic(format!("missing value for {value}"))
                                })?;
                                widget.update_time = digit.parse()?;
                            }
                            "font" => {
                                widget.font = bar_setting_values
                                    .get(ii + 1)
                                    .ok_or_else(|| {
                                        Error::Generic(format!("missing value for {value}"))
                                    })?
                                    .to_string();
                            }
                            "position" => {
                                let val = bar_setting_values
                                    .get(ii + 1)
                                    .ok_or_else(|| {
                                        Error::Generic(format!("{value} is missing a value"))
                                    })?
                                    .to_string();

                                if POSITIONS.contains(&val.as_str()) {
                                    widget.position = val;
                                } else {
                                    return Err(format!("'position' can only have the following values: left, right, middle; {val} is not one of them").into());
                                }
                            }
                            _ => (),
                        }
                    }
                    widget_segment.push(widget);
                }
            }
            "workspace" => {
                if bar_setting_values[0] == "set" {
                    // bar_set 1 set workspace focused_fg "#ffffff" focused_bg "#11ff11" ...
                    let workspace_segment = match &mut bar.workspace_segment {
                        Some(ws) => ws,
                        None => {
                            bar.workspace_segment = Some(WorkspaceSegmentSettings::default());
                            bar.workspace_segment.as_mut().unwrap()
                        }
                    };

                    for (mut ii, value) in bar_setting_values[1..].iter().enumerate() {
                        ii += 1;
                        match &value[..] {
                            "focused_bg" | "focused_background" | "focused_background_color" => {
                                if let Some(next_val) = bar_setting_values.get(ii + 1) {
                                    if !next_val.starts_with('#') {
                                        return Err(format!(
                                            "{next_val} is not a correct value for {value}"
                                        )
                                        .into());
                                    }

                                    workspace_segment.focused_background_color =
                                        next_val.to_string();
                                }
                            }
                            "focused_fg" | "focused_foreground" | "focused_foreground_color" => {
                                if let Some(next_val) = bar_setting_values.get(ii + 1) {
                                    if !next_val.starts_with('#') {
                                        return Err(format!(
                                            "{next_val} is not a correct value for {value}"
                                        )
                                        .into());
                                    }

                                    workspace_segment.focused_foreground_color =
                                        next_val.to_string();
                                }
                            }
                            "normal_bg" | "normal_background" | "normal_background_color" => {
                                if let Some(next_val) = bar_setting_values.get(ii + 1) {
                                    if !next_val.starts_with('#') {
                                        return Err(format!(
                                            "{next_val} is not a correct value for {value}"
                                        )
                                        .into());
                                    }

                                    workspace_segment.normal_background_color =
                                        next_val.to_string();
                                }
                            }
                            "normal_fg" | "normal_foreground" | "normal_foreground_color" => {
                                if let Some(next_val) = bar_setting_values.get(ii + 1) {
                                    if !next_val.starts_with('#') {
                                        return Err(format!(
                                            "{next_val} is not a correct value for {value}"
                                        )
                                        .into());
                                    }

                                    workspace_segment.normal_foreground_color =
                                        next_val.to_string();
                                }
                            }
                            "font" => {
                                workspace_segment.font = bar_setting_values
                                    .get(ii + 1)
                                    .ok_or_else(|| {
                                        Error::Generic(format!("{value} is missing a value"))
                                    })?
                                    .to_string();
                            }
                            "position" => {
                                let val = bar_setting_values
                                    .get(ii + 1)
                                    .ok_or_else(|| {
                                        Error::Generic(format!("{value} is missing a value"))
                                    })?
                                    .to_string();

                                if POSITIONS.contains(&val.as_str()) {
                                    workspace_segment.position = val;
                                } else {
                                    return Err(format!("'position' can only have the following values: left, right, middle; {val} is not one of them").into());
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
            "title" => {
                if &bar_setting_values[0][..] == "set" {
                    // bar_set title set fg "#ffffff" bg "#111111" font "Noto Sans"
                    let title_segment = match &mut bar.title_segment {
                        Some(ws) => ws,
                        None => {
                            bar.title_segment = Some(WindowTitleSettings::default());
                            bar.title_segment.as_mut().unwrap()
                        }
                    };

                    for (mut ii, value) in bar_setting_values[1..].iter().enumerate() {
                        ii += 1;
                        match &value[..] {
                            "font" => {
                                title_segment.font = bar_setting_values
                                    .get(ii + 1)
                                    .ok_or_else(|| {
                                        Error::Generic(format!("{value} is missing a value."))
                                    })?
                                    .to_string();
                            }
                            "foreground_color" | "fg_color" | "fg" => {
                                let val = bar_setting_values
                                    .get(ii + 1)
                                    .ok_or_else(|| {
                                        Error::Generic(format!("{value} is missing a value."))
                                    })?
                                    .to_string();

                                if !val.starts_with('#') {
                                    return Err(format!(
                                        "{val} is not in the correct format, try using."
                                    )
                                    .into());
                                } else {
                                    title_segment.foreground_color = val;
                                }
                            }
                            "background_color" | "bg_color" | "bg" => {
                                let val = bar_setting_values
                                    .get(ii + 1)
                                    .ok_or_else(|| {
                                        Error::Generic(format!("{value} is missing a value."))
                                    })?
                                    .to_string();

                                if !val.starts_with('#') {
                                    return Err(format!(
                                        "{val} is not in the correct format, try using."
                                    )
                                    .into());
                                } else {
                                    title_segment.background_color = val;
                                }
                            }
                            "position" => {
                                let val = bar_setting_values
                                    .get(ii + 1)
                                    .ok_or_else(|| {
                                        Error::Generic(format!("{value} is missing a value"))
                                    })?
                                    .to_string();

                                if POSITIONS.contains(&val.as_str()) {
                                    title_segment.position = val;
                                } else {
                                    return Err(format!("'position' can only have the following values: left, right, middle; {val} is not one of them").into());
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
            "tray" => {
                if &bar_setting_values[0] == "set" {
                    let tray_segment = match &mut bar.icon_tray {
                        Some(it) => it,
                        None => {
                            bar.icon_tray = Some(IconTraySettings::default());
                            bar.icon_tray.as_mut().unwrap()
                        }
                    };
                }
            }
            _ => {
                return Err(
                    format!("bar settings error: no setting {bar_setting_name} exists.").into(),
                )
            }
        }
        Ok(())
    }
}
