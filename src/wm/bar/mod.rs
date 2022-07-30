//! This module contains all the inner workings of the status bar. A status bar is a window
//! containing information useful to the user. Status bar is meant to be shown at almost all times.
//! Bellow is an overview of how the status bar functions.
//!
//! A status bar is constructed from status bar `segments`. A `segment` has a defined type and a
//! position. Segment types are:
//! - Workspace
//! - WindowTitle
//! - Widget
//! - IconTray
//! More information on each segment type can be found in their respective modules bellow.
//!
//! A status bar communicates with the window manager by sending and receiving status bar events.
#![allow(unused)]
pub mod title;
pub mod tray;
pub mod widgets;
pub mod workspace_info;

use std::{cmp::Ordering, fmt::Write};

use cairo::{Context, XCBSurface};
use title::*;
use tray::*;
use widgets::*;
use workspace_info::*;

use crate::{
    config::{BarSettings, SegmentSettings, SegmentSettingsType, WindowTitleSettings},
    errors::{Error, WmResult},
};

use crate::{wm::geometry::Geometry, wm::monitors::MonitorId};

use super::{geometry::TextExtents, workspace::WorkspaceId};

/// Defines where the bar segment should be located within the confines of the bar.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SegmentPosition {
    /// Located on the left most part of the status bar.
    Left,
    /// Located on the right most part of the status bar.
    Right,
    /// Located in the exact middle of the status bar.
    Middle,
}

#[cfg(test)]
mod test {
    use super::SegmentPosition;

    #[test]
    fn check_position_ordering() {
        assert!(
            SegmentPosition::Left < SegmentPosition::Right
                && SegmentPosition::Right < SegmentPosition::Middle
        );
    }
}

impl TryFrom<String> for SegmentPosition {
    type Error = crate::errors::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &value[..] {
            "right" => Ok(Self::Right),
            "left" => Ok(Self::Left),
            "middle" => Ok(Self::Middle),
            _ => Err(format!(
                "Invalid position value: {value}. Possible values are 'right', 'left', 'middle'"
            )
            .into()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SegmentType {
    /// Segment containing information on the workspaces and their names.
    Workspace(WorkspaceInfo),
    /// Window title bar. Since the windows don't have their dedicated title bars, the status bar
    /// is capable of showing the currently focused window title.
    WindowTitle(TitlebarSegment),
    /// Widgets are small compact modules which show system information such as time, date,
    /// battery capacity, CPU or memory utilization and/or others. Widgets can also show some user
    /// defined information. A user can choose how often the widget updates or what actions should
    /// be taken when updating a widget.
    Widget(WidgetSegment),
    /// A place where iconified windows(window icons) will be shown.
    IconTray(IconTraySegment),
}

/// A bar segment is of some type and has a defined position.
#[derive(Debug, Clone)]
pub struct Segment {
    /// Type of the bar segment.
    segment_type: SegmentType,
    /// Position of the bar segment, within the bar.
    /// This field is used when rendering the bar.
    position: SegmentPosition,
}

impl Segment {
    fn draw(&mut self, cr: &Context, position: (f32, f32)) -> WmResult {
        Ok(())
    }

    fn get_drawable_text(&self) -> String {
        match &self.segment_type {
            SegmentType::Widget(widget) => widget.get_text(),
            SegmentType::IconTray(tray) => "[DEBUG]".into(),
            SegmentType::Workspace(ws) => ws.get_text(),
            SegmentType::WindowTitle(title) => title.get_text(),
        }
    }

    fn get_text_extents(&self, cr: &Context, font_size: f64) -> WmResult<TextExtents> {
        match &self.segment_type {
            SegmentType::Widget(widget) => widget.get_text_extents(cr, font_size),
            SegmentType::IconTray(tray) => Ok(TextExtents::default()),
            SegmentType::Workspace(ws) => ws.get_text_extents(cr, font_size),
            SegmentType::WindowTitle(title) => title.get_text_extent(cr, font_size),
        }
    }
}

impl PartialEq for Segment {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl Eq for Segment {}

impl PartialOrd for Segment {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.position.cmp(&other.position))
    }
}

impl Ord for Segment {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.position.cmp(&other.position)
    }
}

impl TryFrom<SegmentSettings> for Segment {
    type Error = Error;
    fn try_from(settings: SegmentSettings) -> Result<Self, Error> {
        match settings.segment_type {
            SegmentSettingsType::Widget(widget_settings) => Ok(Self {
                segment_type: SegmentType::Widget(WidgetSegment::from(widget_settings)),
                position: SegmentPosition::try_from(settings.position)?,
            }),
            SegmentSettingsType::Workspace(workspace_settings) => Ok(Self {
                segment_type: SegmentType::Workspace(WorkspaceInfo::from(workspace_settings)),
                position: SegmentPosition::try_from(settings.position)?,
            }),
            SegmentSettingsType::Title(window_title_settings) => Ok(Self {
                segment_type: SegmentType::WindowTitle(TitlebarSegment::from(
                    window_title_settings,
                )),
                position: SegmentPosition::try_from(settings.position)?,
            }),
            SegmentSettingsType::IconTray(icon_tray) => Ok(Self {
                segment_type: SegmentType::IconTray(IconTraySegment::from(icon_tray)),
                position: SegmentPosition::try_from(settings.position)?,
            }),
        }
    }
}

/// A bar consists of a list of segments.
#[derive(Default, Clone, Debug)]
pub struct Bar {
    /// All the segments located in a single bar window.
    segments: Vec<Segment>,
    /// Unique identifier of the bar.
    id: u32,
    /// Identifier of the monitor this bar is located on.
    monitor: MonitorId,
    /// X11 window id of the bar window.
    window_id: Option<u32>,
    /// Cairo surface
    surface: Option<XCBSurface>,
    /// Size of the bar window.
    geometry: Option<Geometry>,
    /// Settings.
    settings: Option<BarSettings>,
}

impl Bar {
    /// Create a new bar.
    pub fn new<U: Into<u32>>(id: U, monitor: U, bar_settings: &BarSettings) -> WmResult<Self> {
        let mut segments = Vec::new();
        for segment in bar_settings.segments.iter() {
            let segment = segment.clone().try_into()?;
            segments.push(segment);
        }

        Ok(Self {
            id: id.into(),
            monitor: monitor.into(),
            segments,
            window_id: None,
            surface: None,
            geometry: None,
            settings: Some(bar_settings.clone()),
        })
    }

    pub fn settings(&self) -> WmResult<&BarSettings> {
        self.settings
            .as_ref()
            .ok_or_else(|| Error::Generic("this bar has no settings!".into()))
    }

    /// Get the id of the bar.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Get the monitor id of the bar.
    pub fn monitor(&self) -> u32 {
        self.monitor
    }

    /// Get the X window id of the bar window.
    pub fn window_id(&self) -> WmResult<u32> {
        self.window_id
            .ok_or_else(|| Error::Generic("bar does not have an associated window id.".to_string()))
    }

    /// Get the reference to the windows Cairo surface.
    pub fn surface(&self) -> WmResult<&XCBSurface> {
        self.surface
            .as_ref()
            .ok_or_else(|| Error::Generic("bar does not have a surface.".to_string()))
    }

    /// Get a copy of the window geometry.
    pub fn geometry(&self) -> WmResult<Geometry> {
        self.geometry
            .ok_or_else(|| Error::Generic("bar does not have a known geometry.".to_string()))
    }

    /// Set the X11 window id of the bar window.
    pub fn set_window_id<I: Into<u32>>(&mut self, wid: I) {
        self.window_id = Some(wid.into())
    }

    /// Set the bar's Cairo surface.
    pub fn set_surface(&mut self, surface: XCBSurface) {
        self.surface = Some(surface)
    }

    /// Set the bar's geometry.
    pub fn set_geometry(&mut self, geometry: Geometry) {
        self.geometry = Some(geometry)
    }

    /// Get the latest values for the bar.
    pub fn update(
        &mut self,
        focused_workspace: Option<WorkspaceId>,
        open_workspace: Option<WorkspaceId>,
        window_title: String,
    ) -> WmResult {
        self.update_widgets()?;
        self.update_workspace_info(focused_workspace, open_workspace)?;
        self.update_window_title(window_title); // TODO Maybe change this so that
        Ok(())
    }

    /// Redraw the entire bar.
    pub fn redraw(&mut self) -> WmResult {
        let geom = self.geometry()?;
        let cr = Context::new(self.surface()?)?;
        cr.set_operator(cairo::Operator::Clear);
        cr.rectangle(0.0, 0.0, geom.width.into(), geom.height.into());
        cr.fill()?;
        cr.set_operator(cairo::Operator::Source);

        let mut sorted = self.segments.clone();
        sorted.sort();

        let (left_extents, middle_extents, right_extents) = self.get_bar_text_extents(&cr)?;

        Ok(())
    }

    fn get_bar_text_extents(
        &self,
        cr: &Context,
    ) -> WmResult<(TextExtents, TextExtents, TextExtents)> {
        let mut sorted = self.segments.clone();
        sorted.sort();

        let mut left_extents = TextExtents::default();
        let mut middle_extents = TextExtents::default();
        let mut right_extents = TextExtents::default();
        let size = self.settings()?.font_size as _;

        for segment in sorted.iter_mut() {
            match segment.position {
                SegmentPosition::Left => {
                    left_extents += segment.get_text_extents(&cr, size)?;
                }
                SegmentPosition::Middle => {
                    middle_extents += segment.get_text_extents(&cr, size)?;
                }
                SegmentPosition::Right => {
                    right_extents += segment.get_text_extents(&cr, size)?;
                }
            }
        }

        Ok((left_extents, middle_extents, right_extents))
    }

    pub fn get_height(&self) -> WmResult<u32> {
        let cr = Context::new(self.surface()?)?;
        let (left_extents, middle_extents, right_extents) = self.get_bar_text_extents(&cr)?;

        let ret = [
            left_extents.height,
            middle_extents.height,
            right_extents.height,
        ]
        .into_iter()
        .reduce(f64::max)
        .ok_or_else(|| {
            Error::Generic("Unable to get the bar height, using the default value".into())
        })?;

        println!("height: {ret}");
        Ok(ret as _)
    }

    /// Adding workspace info to the bar, based on which monitor it is located.
    ///
    /// This adds the workspace info to every workspace segment in the bar.
    pub fn create_workspaces(&mut self, workspace_ids: Vec<(String, u32)>) {
        let mut segments: Vec<&mut Segment> = self
            .segments
            .iter_mut()
            .filter(|segment| matches!(segment.segment_type, SegmentType::Workspace(_)))
            .collect();
        for tuple in workspace_ids.iter() {
            let segment = WorkspaceInfoSegment::new(tuple.0.clone(), tuple.1);
            for workspace_info in segments.iter_mut() {
                if let SegmentType::Workspace(info) = &mut workspace_info.segment_type {
                    info.add(segment.clone())
                }
            }
        }
    }

    /// Try to update every widget.
    ///
    /// A widget is only updated(by running its associated command) when the time between now and the last update
    /// is greater than the `update_interval` widget setting.
    pub fn update_widgets(&mut self) -> WmResult {
        let mut segments: Vec<&mut Segment> = self
            .segments
            .iter_mut()
            .filter(|segment| matches!(segment.segment_type, SegmentType::Widget(_)))
            .collect();

        for segment in segments.iter_mut() {
            if let SegmentType::Widget(widgets) = &mut segment.segment_type {
                widgets.run_updates()?;
            }
        }
        Ok(())
    }

    fn update_workspace_info(
        &mut self,
        focused_workspace: Option<WorkspaceId>,
        open_workspace: Option<WorkspaceId>,
    ) -> WmResult {
        let mut segments: Vec<&mut Segment> = self
            .segments
            .iter_mut()
            .filter(|segment| matches!(segment.segment_type, SegmentType::Workspace(_)))
            .collect();

        for segment in segments.iter_mut() {
            if let SegmentType::Workspace(workspace_info) = &mut segment.segment_type {
                workspace_info.set_focused(focused_workspace);
                workspace_info.set_open(open_workspace);
            }
        }

        Ok(())
    }

    fn update_window_title(&mut self, window_title: String) {
        let mut segments: Vec<&mut Segment> = self
            .segments
            .iter_mut()
            .filter(|segment| matches!(segment.segment_type, SegmentType::WindowTitle(_)))
            .collect();

        for segment in segments.iter_mut() {
            if let SegmentType::WindowTitle(title_segment) = &mut segment.segment_type {
                title_segment.set_title(window_title.clone())
            }
        }
    }
}

pub enum BarEvent {
    ButtonPress,
    ButtonRelease,
    KeyPress,
    KeyPressRelease,
    WorkspaceChange,
    IconChange,
    WidgetChange,
    TitleChange,
}