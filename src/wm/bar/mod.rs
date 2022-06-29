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

use title::*;
use tray::*;
use widgets::*;
use workspace_info::*;

use super::monitors::MonitorId;

/// Defines where the bar segment should be located within the confines of the bar.
pub enum SegmentPosition {
    /// Located on the left most part of the status bar.
    Left,
    /// Located on the right most part of the status bar.
    Right,
    /// Located in the exact middle of the status bar.
    Middle,
}

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
pub struct Segment {
    segment_type: SegmentType,
    position: SegmentPosition,
    monitor: MonitorId,
}

/// A bar consists of a list of segments.
pub struct Bar {
    segments: Vec<Segment>,
    id: u32,
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
