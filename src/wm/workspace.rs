use std::cell::RefCell;
use std::rc::Rc;

use super::geometry::Geometry;
use super::layouts::{Layout, LayoutType};
use crate::errors::WmResult;

use super::container::{Client, ContainerId, ContainerList, ContainerListNode};

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

    pub fn insert(&mut self, c: Client) -> WmResult<ContainerId> {
        self.containers.add_front(c)
    }

    pub fn insert_many(&mut self, cs: Vec<Client>) -> WmResult<Vec<ContainerId>> {
        let mut ret = Vec::new();
        for c in cs {
            ret.push(self.containers.add_front(c)?);
        }

        Ok(ret)
    }

    pub fn apply_layout(&mut self, screen: Geometry) -> WmResult {
        self.layout.apply(screen, self.containers.get_all())
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

    pub(crate) fn find<I: Into<ContainerId> + Copy>(
        &self,
        id: I,
    ) -> WmResult<Rc<RefCell<ContainerListNode>>> {
        self.containers.find(id)
    }

    pub(crate) fn find_many<I: Into<ContainerId> + Copy>(
        &self,
        ids: Vec<I>,
    ) -> Vec<Option<(u32, Rc<RefCell<ContainerListNode>>)>> {
        let mut ret = Vec::with_capacity(ids.len());

        for id in ids {
            let node_result = self.containers.find(id);
            if node_result.is_ok() {
                let n = node_result.as_ref().unwrap().clone();
                if let Ok(node) = n.try_borrow() {
                    ret.push(Some((node.data().wid().unwrap(), node_result.unwrap())))
                };
            } else {
                ret.push(None)
            }
        }

        ret
    }

    pub(crate) fn get_all(&self) -> WmResult<Vec<Rc<RefCell<ContainerListNode>>>> {
        Ok(self.containers.get_all())
    }
}

pub type WorkspaceId = u32;

pub type Workspaces = Vec<Workspace>;
