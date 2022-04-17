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

    /// Contains a client with the given process id?
    pub fn _contains_pid(&self, pid: u32) -> bool {
        self.containers.id_for_pid(pid).is_ok()
    }

    /// Contains a client with the given container id?
    pub fn _contains<I: Into<ContainerId> + Copy>(&self, id: I) -> bool {
        self.containers.find(id.into()).is_ok()
    }

    pub fn insert(&mut self, c: Client, t: u8) -> ContainerId {
        self.containers.insert_front(c, t)
    }

    pub fn insert_many(&mut self, cs: Vec<Client>, t: Vec<u8>) -> Vec<ContainerId> {
        let mut ret = Vec::new();
        for (i, &c) in cs.iter().enumerate() {
            ret.push(self.containers.insert_front(c, t[i]));
        }

        ret
    }

    pub fn apply_layout(&mut self, screen: Geometry) -> WmResult {
        self.layout.apply(screen, self.containers.get_all_mut())
    }

    pub fn remove_wid(&mut self, wid: u32) -> WmResult {
        if let Ok(id) = self.containers.id_for_wid(wid) {
            self.containers.remove(id)?;
        };

        Ok(())
    }

    pub fn _remove_pid(&mut self, wid: u32) -> WmResult {
        if let Ok(id) = self.containers.id_for_pid(wid) {
            self.containers.remove(id)?;
        };

        Ok(())
    }

    pub(crate) fn find<I: Into<ContainerId> + Copy>(&self, id: I) -> WmResult<&Container> {
        self.containers.find(id.into())
    }

    pub(crate) fn find_many<I: Into<ContainerId> + Copy>(&self, ids: Vec<I>) -> WmResult<Vec<&Container>> {
        let mut ret = Vec::new();

        for id in ids {
            if let Ok(c) = self.containers.find(id.into()) {
                ret.push(c)
            }
        }

        Ok(ret)
    }

    pub(crate) fn get_all(&self) -> WmResult<std::collections::vec_deque::Iter<Container>> {
        Ok(self.containers.get_all())
    }
}

pub type WorkspaceId = u32;

pub type Workspaces = Vec<Workspace>;
