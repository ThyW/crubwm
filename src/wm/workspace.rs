use std::rc::Rc;

use x11rb::rust_connection::RustConnection;

use super::geometry::Geometry;
use super::layouts::{Layout, LayoutType};
use crate::errors::WmResult;

use super::container::{Client, Container, ContainerId, ContainerList};

pub struct Workspace {
    pub containers: ContainerList,
    layout: LayoutType,
    allowed_layouts_mask: u64,
    pub name: String,
    pub id: WorkspaceId,
}

impl Workspace {
    pub fn new(name: String, id: WorkspaceId, allowed_layouts_mask: u64) -> Self {
        Self {
            containers: ContainerList::new(id),
            layout: LayoutType::default(),
            allowed_layouts_mask,
            name,
            id,
        }
    }

    pub fn change_layout(&mut self, layout_string: String) -> WmResult {
        self.layout = LayoutType::try_from(layout_string.as_str())?;

        Ok(())
    }

    pub fn cycle_layout(&mut self) -> WmResult {
        if self.allowed_layouts_mask == 0 {
            return Err("workspace error: no layouts are available for this workspace.".into());
        }
        let n = self.layout as u64;
        let mut active_layotus = Vec::new();

        for ii in 0..64 {
            if self.allowed_layouts_mask & 1 << ii != 0 {
                active_layotus.push(1 << ii)
            }
        }

        for (index, layout) in active_layotus.iter().enumerate() {
            if *layout == n {
                if let Some(next) = active_layotus.get(index + 1) {
                    self.layout = LayoutType::try_from(*next)?;
                    return Ok(());
                } else if active_layotus.len() > 1 {
                    self.layout = LayoutType::try_from(active_layotus[0])?;
                    return Ok(());
                } else {
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    /// Contains a client with the given window id?
    pub fn contains_window(&self, wid: u32) -> bool {
        self.containers.id_for_window(wid).is_ok()
    }

    pub fn find_by_window_id(&self, wid: u32) -> WmResult<&Container> {
        let id = self.containers.id_for_window(wid)?;
        self.find(id)
    }
    pub fn find_by_window_id_mut(&mut self, wid: u32) -> WmResult<&mut Container> {
        let id = self.containers.id_for_window(wid)?;
        self.find_mut(id)
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

    pub fn apply_layout(&mut self, screen: Geometry, connection: Rc<RustConnection>) -> WmResult {
        self.layout
            .apply(screen, self.containers.iter_in_layout_mut(), connection)
    }

    pub fn remove_window(&mut self, wid: u32) -> WmResult {
        if let Ok(id) = self.containers.id_for_window(wid) {
            self.containers.remove(id)?;
        };

        Ok(())
    }

    pub fn remove_and_return_window(&mut self, wid: u32) -> WmResult<Container> {
        if let Ok(id) = self.containers.id_for_window(wid) {
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
        self.containers.previous_for_id(c)
    }

    pub fn find<I: Into<ContainerId>>(&self, id: I) -> WmResult<&Container> {
        self.containers.find(id)
    }

    pub fn find_mut<I: Into<ContainerId>>(&mut self, id: I) -> WmResult<&mut Container> {
        self.containers.find_mut(id)
    }

    pub fn iter_containers(&self) -> WmResult<std::collections::vec_deque::Iter<Container>> {
        Ok(self.containers.iter())
    }

    pub fn insert_container(&mut self, container: Container) -> WmResult<ContainerId> {
        self.containers.insert_back_full(container)
    }
}

pub type WorkspaceId = u32;

pub type Workspaces = Vec<Workspace>;
