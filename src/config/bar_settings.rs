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
    /// Foreground color of the icon text.
    pub icon_color: String,
    /// Foreground color of the value text.
    pub value_color: String,
    /// Foreground color of the value text.
    pub separator_color: String,
    /// Background color for the whole widget.
    pub background_color: String,
    /// A command that is run on every update.
    pub command: String,
    /// Time, in seconds, of how often should the widget be updated.
    pub update_time: u32,
    /// Font of the widget.
    pub font: String,
    /// Text which separates two widgets from one another.
    pub separator: String,
}

impl Default for WidgetSettings {
    fn default() -> Self {
        Self {
            id: "none".into(),
            icon: "NONE".into(),
            icon_color: "#ffffff".into(),
            value_color: "#ffffff".into(),
            separator_color: "#ffffff".into(),
            background_color: "#00a2ff".into(),
            command: "".into(),
            update_time: 0,
            font: "monospace".into(),
            separator: "|".into(),
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
}

impl Default for WorkspaceSegmentSettings {
    fn default() -> Self {
        Self {
            focused_foreground_color: "#ffffff".to_string(),
            focused_background_color: "#00a2ff".to_string(),
            normal_foreground_color: "#ffffff".to_string(),
            normal_background_color: "#333333".to_string(),
            font: "monospace".to_string(),
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
}

impl Default for WindowTitleSettings {
    fn default() -> Self {
        Self {
            font: "monospace".into(),
            foreground_color: "#ffffff".into(),
            background_color: "#00a2ff".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct IconTraySettings {}

impl Default for IconTraySettings {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone)]
pub struct BarSettings {
    /// identifier for the bar, unique 32-bit integer
    pub identifier: u32,
    /// monitor id of the bar
    pub monitor: u32,
    /// All the widget settings for the given monitor.
    pub segments: Vec<SegmentSettings>,
    /// Size of all fonts used in the bar.
    pub font_size: u32,
    /// Height of the bar.
    pub height: u32,
    /// Background color of the bar.
    pub background_color: String,
}

impl BarSettings {
    pub fn contains_tray(&self) -> bool {
        for segment in self.segments.iter() {
            if matches!(segment.segment_type, SegmentSettingsType::Title(_)) {
                return true
            }
        }

        false
    }
}

#[derive(Debug, Clone)]
pub struct SegmentSettings {
    pub segment_type: SegmentSettingsType,
    pub position: String,
    pub name: String,
}

impl SegmentSettings {
    fn new(segment_type: SegmentSettingsType, position: String, name: String) -> Self {
        Self {
            segment_type,
            position,
            name,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SegmentSettingsType {
    Widget(Vec<WidgetSettings>),
    Workspace(WorkspaceSegmentSettings),
    Title(WindowTitleSettings),
    IconTray(IconTraySettings),
}

impl BarSettings {
    fn new(identifier: u32) -> Self {
        Self {
            background_color: "#333333".into(),
            identifier,
            monitor: 1,
            segments: Vec::new(),
            font_size: 10,
            height: 15,
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
            "segment" => {
                if &bar_setting_values[0] == "add" {
                    match &bar_setting_values.get(1).ok_or_else(|| Error::Generic("missing new segment type".into()))?[..] {
                        // bar_set 0 segment add "widget" "wiget1" "right"
                        "widget" => {
                            let name = bar_setting_values.get(2).ok_or_else(|| Error::Generic("missing new  widget segment name".into()))?;
                            if let Ok(position_value) = bar_setting_values.get(3).ok_or_else(|| Error::Generic("Missing position specification for new segment.".into())) {
                                if POSITIONS.contains(&position_value.as_str()) {
                                    let widget_segment = SegmentSettings::new(SegmentSettingsType::Widget(Default::default()), position_value.clone(), name.clone());
                                    bar.segments.push(widget_segment);
                                }
                            }
                        }
                        "workspace" => {
                            let name = bar_setting_values.get(2).ok_or_else(|| Error::Generic("missing new workspace segment name".into()))?;
                            if let Ok(position_value) = bar_setting_values.get(3).ok_or_else(|| Error::Generic("Missing position specification for new segment.".into())) {
                                if POSITIONS.contains(&position_value.as_str()) {
                                    let widget_segment = SegmentSettings::new(SegmentSettingsType::Workspace(Default::default()), position_value.clone(), name.clone());
                                    bar.segments.push(widget_segment);
                                }
                            }
                        }
                        "title" => {
                            let name = bar_setting_values.get(2).ok_or_else(|| Error::Generic("missing new window title segment name".into()))?;
                            if let Ok(position_value) = bar_setting_values.get(3).ok_or_else(|| Error::Generic("Missing position specification for new segment.".into())) {
                                if POSITIONS.contains(&position_value.as_str()) {
                                    let widget_segment = SegmentSettings::new(SegmentSettingsType::Title(Default::default()), position_value.clone(), name.clone());
                                    bar.segments.push(widget_segment);
                                }
                            }
                        }
                        "icon_tray" => {
                            let name = bar_setting_values.get(2).ok_or_else(|| Error::Generic("missing new icon tray segment name".into()))?;
                            if let Ok(position_value) = bar_setting_values.get(3).ok_or_else(|| Error::Generic("Missing position specification for new segment.".into())) {
                                if POSITIONS.contains(&position_value.as_str()) {
                                    let widget_segment = SegmentSettings::new(SegmentSettingsType::IconTray(Default::default()), position_value.clone(), name.clone());
                                    bar.segments.push(widget_segment);
                                }
                            }
                        }
                        x => return Err(format!("{x} is not recognized as a valid bar segment type.\nValid segment types are: 'widget', 'workspace', 'window_title', 'icon_tray'.").into())
                    }
                }
            }
            "monitor" => {
                bar.monitor = bar_setting_values[0].parse::<u32>()?;
            }
            "widget" => {
                if &bar_setting_values[0] == "add" {
                    // bar_set 0 widget add "battery" icon "" command "acpi" font "Iosevka" update_time "5"
                    let name = &bar_setting_values[1];

                    let widget_segment = bar
                        .segments
                        .iter_mut()
                        .find(|x| &x.name == name)
                        .ok_or_else(|| {
                            Error::Generic(format!("Widget segment {name} does not exist"))
                        })?;
                    let name = bar_setting_values[1].clone();
                    let mut widget = WidgetSettings::default();
                    for (mut ii, value) in bar_setting_values[2..].iter().enumerate() {
                        ii += 2;
                        match &value[..] {
                            "icon" => {
                                widget.icon = bar_setting_values
                                    .get(ii + 1)
                                    .ok_or_else(|| {
                                        Error::Generic(format!("missing value for {value}"))
                                    })?
                                    .to_string();
                            }
                            "icon_fg" | "icon_foreground" => {
                                if let Some(next_val) = bar_setting_values.get(ii + 1) {
                                    if !next_val.starts_with('#') {
                                        return Err(format!(
                                            "{next_val} is not a correct value for {value}"
                                        )
                                        .into());
                                    }

                                    widget.icon_color = next_val.to_string();
                                }
                            }
                            "value_fg" | "value_foreground" => {
                                if let Some(next_val) = bar_setting_values.get(ii + 1) {
                                    if !next_val.starts_with('#') {
                                        return Err(format!(
                                            "{next_val} is not a correct value for {value}"
                                        )
                                        .into());
                                    }

                                    widget.value_color = next_val.to_string();
                                }
                            }
                            "separator_fg" | "separator_foreground" => {
                                if let Some(next_val) = bar_setting_values.get(ii + 1) {
                                    if !next_val.starts_with('#') {
                                        return Err(format!(
                                            "{next_val} is not a correct value for {value}"
                                        )
                                        .into());
                                    }

                                    widget.separator_color = next_val.to_string();
                                }
                            }
                            "bg" | "bg_color" | "background_color" => {
                                if let Some(next_val) = bar_setting_values.get(ii + 1) {
                                    if !next_val.starts_with('#') {
                                        return Err(format!(
                                            "{next_val} is not a correct value for {value}"
                                        )
                                        .into());
                                    }

                                    widget.background_color = next_val.to_string();
                                }
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
                            "separator" => {
                                widget.separator = bar_setting_values
                                    .get(ii + 1)
                                    .ok_or_else(|| {
                                        Error::Generic(format!("missing value for {value}"))
                                    })?
                                    .to_string();
                            }
                            _ => (),
                        }
                    }
                    if let SegmentSettingsType::Widget(widgets) = &mut widget_segment.segment_type {
                        widgets.push(widget)
                    }
                }
            }
            "workspace" => {
                if bar_setting_values[0] == "set" {
                    // bar_set 1 set workspace focused_fg "#ffffff" focused_bg "#11ff11" ...
                    let name = &bar_setting_values[1];
                    let workspace_segment = bar
                        .segments
                        .iter_mut()
                        .find(|x| &x.name == name)
                        .ok_or_else(|| {
                            Error::Generic(format!("Unable to find workspace segment {name}"))
                        })?;

                    if let SegmentSettingsType::Workspace(workspace_segment) =
                        &mut workspace_segment.segment_type
                    {
                        for (mut ii, value) in bar_setting_values[2..].iter().enumerate() {
                            ii += 2;
                            match &value[..] {
                                "focused_bg"
                                | "focused_background"
                                | "focused_background_color" => {
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
                                "focused_fg"
                                | "focused_foreground"
                                | "focused_foreground_color" => {
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
                                _ => (),
                            }
                        }
                    }
                }
            }
            "title" => {
                if &bar_setting_values[0][..] == "set" {
                    // bar_set title set fg "#ffffff" bg "#111111" font "Noto Sans"
                    let name = &bar_setting_values[1];

                    let title_segment = bar
                        .segments
                        .iter_mut()
                        .find(|x| &x.name == name)
                        .ok_or_else(|| {
                            Error::Generic(format!("Unable to find segment with name {name}"))
                        })?;

                    if let SegmentSettingsType::Title(title_segment) =
                        &mut title_segment.segment_type
                    {
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
                                _ => (),
                            }
                        }
                    }
                }
            }
            "icon_tray" | "tray" => if &bar_setting_values[0] == "set" {},
            "font_size" => bar.font_size = bar_setting_values[0].parse()?,
            "height" => bar.height = bar_setting_values[0].parse()?,
            "background_color" => {
                let val = bar_setting_values[0].clone();
                if !val.starts_with('#') {
                    return Err(format!("{val} is not a valid color format!").into());
                }
                bar.background_color = val;
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

impl IntoIterator for AllBarSettings {
    type Item = BarSettings;

    type IntoIter = std::vec::IntoIter<BarSettings>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
