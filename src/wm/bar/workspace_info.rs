use crate::wm::workspace::WorkspaceId;

/// The workspace info segment informs the user about the current state of the window manager's
/// workspaces. It shows information such as the workspaces available for the current monitor,
/// the focused workspace, workspace names and urgent workspaces.
#[derive(Debug)]
pub struct WorkspaceInfoSegment {
    /// Name of the workspace/what is displayed.
    name: String,
    /// Workspace number or id.
    workspace_id: WorkspaceId,
    /// Is the workspace focused?
    focused: bool,
    /// Does the workspace seek urgent attention?
    urgent: bool,
    /// Default color of the workspace.
    color: Option<u32>,
}

/// The workspace info consists of different workspace info segments.
pub struct WorkspaceInfo {
    workspaces: Vec<WorkspaceInfoSegment>,
}
