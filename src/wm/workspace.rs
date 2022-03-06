use super::container::ContainerList;

pub struct Workspace {
    containers: ContainerList,
    pub name: String,
    pub id: WorkspaceId,
}

pub type WorkspaceId = u32;

pub type Workspaces = Vec<Workspace>;
