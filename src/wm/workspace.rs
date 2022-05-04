use super::geometry::Geometry;
use super::layouts::{Layout, LayoutType};
use crate::errors::WmResult;

use super::container::{Client, Container, ContainerId, ContainerList};

pub struct Workspace {
    pub containers: ContainerList,
    layout: LayoutType,
    pub name: String,
    pub id: WorkspaceId,
}

impl Workspace {
    pub fn new(name: String, id: WorkspaceId) -> Self {
        Self {
            containers: ContainerList::new(id),
            layout: LayoutType::default(),
            name,
            id,
        }
    }

    /// Contains a client with the given window id?
    pub fn contains_wid(&self, wid: u32) -> bool {
        self.containers.id_for_wid(wid).is_ok()
    }

    pub fn find_by_window_id(&self, wid: u32) -> WmResult<&Container> {
        let id = self.containers.id_for_wid(wid)?;
        self.find(id)
    }

    pub fn insert_client(&mut self, c: Client, t: u8) -> ContainerId {
        self.containers.insert_back(c, t)
    }

    pub fn insert_many(&mut self, cs: Vec<Client>, t: Vec<u8>) -> Vec<ContainerId> {
        let mut ret = Vec::new();
        for (i, &c) in cs.iter().enumerate() {
            ret.push(self.containers.insert_back(c, t[i]));
        }

        ret
    }

    pub fn apply_layout(&mut self, screen: Geometry) -> WmResult {
        self.layout.apply(screen, self.containers.get_all_mut())
    }

    pub fn remove_window(&mut self, wid: u32) -> WmResult {
        if let Ok(id) = self.containers.id_for_wid(wid) {
            self.containers.remove(id)?;
        };

        Ok(())
    }

    pub fn remove_and_return_window(&mut self, wid: u32) -> WmResult<Container> {
        if let Ok(id) = self.containers.id_for_wid(wid) {
            return self.containers.remove(id);
        }

        Err(crate::errors::Error::Generic(format!(
            "workspace error: unable to find client with window id {wid} to remove"
        )))
    }

    pub fn next_container(&self, c: ContainerId) -> WmResult<&Container> {
        self.containers.next_for_id(c)
    }
    pub fn previous_container(&self, c: ContainerId) -> WmResult<&Container> {
        self.containers.prev_for_id(c)
    }

    pub fn find<I: Into<ContainerId> + Copy>(&self, id: I) -> WmResult<&Container> {
        self.containers.find(id.into())
    }

    pub fn iter_containers(&self) -> WmResult<std::collections::vec_deque::Iter<Container>> {
        Ok(self.containers.get_all())
    }

    pub fn insert_container(&mut self, container: Container) -> WmResult<ContainerId> {
        self.containers.insert_back_full(container)
    }
}

pub type WorkspaceId = u32;

pub type Workspaces = Vec<Workspace>;
