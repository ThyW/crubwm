#![allow(unused)]
use x11rb::protocol::randr::Output;

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
    focused_workspace: Option<WorkspaceId>,
}

impl Monitor {
    pub fn new(size: Geometry, id: MonitorId, outputs: Vec<Output>) -> Self {
        Self {
            size,
            id,
            outputs,
            workspaces: Vec::new(),
            focused_workspace: None,
        }
    }

    pub fn add_workspace(&mut self, workspace: WorkspaceId) {
        if self.workspaces.contains(&workspace) {
            return;
        }

        self.workspaces.push(workspace)
    }

    pub fn get_focused_workspace_id(&self) -> WmResult<WorkspaceId> {
        if let Some(fw) = self.focused_workspace {
            return Ok(fw);
        }

        Err("monitor: This monitor does not have any focused workspace.".into())
    }

    pub fn set_focused_workspace(&mut self, id: WorkspaceId) {
        self.focused_workspace = Some(id)
    }
}
