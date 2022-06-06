#![allow(unused)]
use x11rb::protocol::randr::{MonitorInfo, Output};

use crate::{
    errors::WmResult,
    wm::{geometry::Geometry, workspace::WorkspaceId},
};

pub type MonitorId = u32;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Monitor {
    size: Geometry,
    id: MonitorId,
    outputs: Vec<Output>,
    workspaces: Vec<WorkspaceId>,
    open_workspace: Option<WorkspaceId>,
    focused: bool,
}

impl Monitor {
    fn new(size: Geometry, id: MonitorId, outputs: Vec<Output>) -> Self {
        Self {
            size,
            id,
            outputs,
            workspaces: Vec::new(),
            open_workspace: None,
            focused: false,
        }
    }

    pub fn from_monitor_info<I: Into<MonitorId>>(info: MonitorInfo, id: I) -> WmResult<Self> {
        let size = Geometry {
            x: info.x,
            y: info.y,
            width: info.width,
            height: info.height,
        };
        let outputs = info.outputs;

        Ok(Self::new(size, id.into(), outputs))
    }

    pub fn add_workspace(&mut self, workspace: WorkspaceId) {
        if self.workspaces.contains(&workspace) {
            return;
        }

        self.workspaces.push(workspace)
    }

    pub fn get_open_workspace(&self) -> WmResult<WorkspaceId> {
        if let Some(id) = self.open_workspace {
            return Ok(id);
        }

        Err("This monitor does not have any open workspaces.".into())
    }

    pub fn set_open_workspace(&mut self, id: Option<WorkspaceId>) -> WmResult {
        if let Some(new_id) = id {
            if !self.contains(&new_id) {
                return Err("This workspace is not located in on this monitor.".into());
            }
            self.open_workspace = Some(new_id);
            Ok(())
        } else if let Some(workspace_id) = self.workspaces.get(0) {
            self.open_workspace = Some(*workspace_id);
            Ok(())
        } else {
            Err("This monitor does not have any workspaces.".into())
        }
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn focus(&mut self, focus: bool) {
        self.focused = focus;
    }

    pub fn size(&self) -> Geometry {
        self.size
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn contains(&self, workspace: &u32) -> bool {
        self.workspaces.contains(workspace)
    }
}
