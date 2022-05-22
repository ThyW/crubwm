use std::rc::Rc;

use super::focus_stack::FocusStack;
use super::geometry::Geometry;
use super::layouts::{Layout, LayoutType};
use crate::errors::WmResult;

use super::container::{Client, Container, ContainerId, ContainerList};

#[derive(Clone)]
pub struct Workspace {
    containers: ContainerList,
    layout: LayoutType,
    allowed_layouts_mask: u64,
    screen_size: Geometry,
    pub name: String,
    pub id: WorkspaceId,
    pub focus: FocusStack,
}

impl Workspace {
    /// Create a new workspace.
    pub fn new(
        name: String,
        id: WorkspaceId,
        allowed_layouts_mask: u64,
        root_window: u32,
        screen_size: Geometry,
    ) -> Self {
        Self {
            containers: ContainerList::new(id),
            layout: LayoutType::default(),
            allowed_layouts_mask,
            name,
            id,
            focus: FocusStack::new(root_window),
            screen_size,
        }
    }

    /// Change the current workspace layout, given a string identifying the new layout.
    pub fn change_layout(&mut self, layout_string: String) -> WmResult {
        let layout = LayoutType::try_from(layout_string.as_str())?;
        if layout as u64 & self.allowed_layouts_mask == 1 {
            self.layout = layout
        }

        Ok(())
    }

    /// Switch to the next layout from the allowed layout mask.
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

    /// Given an X window id, return an immutable reference to the `Container` which holds a client
    /// that has this window id.
    pub fn find_by_window_id(&self, wid: u32) -> WmResult<&Container> {
        let id = self.containers.id_for_window(wid)?;
        self.find(id)
    }

    /// Given an X window id, return a mutable reference to the `Container` which holds a client
    /// that has this window id.
    pub fn find_by_window_id_mut(&mut self, wid: u32) -> WmResult<&mut Container> {
        let id = self.containers.id_for_window(wid)?;
        self.find_mut(id)
    }

    /// Insert a client into the workspace, given a `Client` and the container type mask.
    pub fn insert_client(&mut self, c: Client, t: u8) -> ContainerId {
        self.containers.insert_back(c, t)
    }

    /// Insert multiple clients into the workspace, given an `Iterator` over `Client`s and an
    /// `Iterator` over container type masks.
    pub fn insert_many<C: std::iter::Iterator<Item = Client>, T: std::iter::Iterator<Item = u8>>(
        &mut self,
        cs: C,
        t: T,
    ) -> Vec<ContainerId> {
        let mut ret = Vec::new();
        for (c, i) in cs.zip(t) {
            ret.push(self.containers.insert_back(c, i));
        }

        ret
    }

    /// This function takes the size of the screen, or a workspace and applies the layout rules to
    /// all of the container's workspaces. To achieve this, we also require the X connection, so
    /// that the layout rules for the clients can be applied right away.
    pub fn apply_layout<C: x11rb::connection::Connection>(
        &mut self,
        connection: Rc<C>,
        screen_size: Option<Geometry>,
    ) -> WmResult {
        let screen_size = screen_size.unwrap_or(self.screen_size);
        self.layout.apply(
            screen_size,
            self.containers.iter_in_layout_mut(),
            connection,
        )
    }

    /// Attempt to remove a `Container` with the given window id.
    pub fn remove_window(&mut self, wid: u32) -> WmResult {
        if let Ok(id) = self.containers.id_for_window(wid) {
            self.containers.remove(id)?;
        };

        Ok(())
    }

    /// Attempt to remove a `Container` with the given window id, while also returning it.
    ///
    /// This function is used for moving `Container`s between workspaces.
    pub fn remove_and_return_window(&mut self, wid: u32) -> WmResult<Container> {
        if let Ok(id) = self.containers.id_for_window(wid) {
            return self.containers.remove(id);
        }

        Err(crate::errors::Error::Generic(format!(
            "workspace error: unable to find client with window id {wid} to remove"
        )))
    }

    /// Attempt to return a reference to the next container in the `ContainerList` belonging to
    /// this workspace.
    pub fn next_container(&self, c: ContainerId) -> WmResult<&Container> {
        self.containers.next_for_id(c)
    }

    /// Attempt the return a reference to the previous container in the `ContainerList` belonging
    /// to this workspace.
    pub fn previous_container(&self, c: ContainerId) -> WmResult<&Container> {
        self.containers.previous_for_id(c)
    }

    /// Attempt to find a `Container` given its `ContainerId`, returning an immutable reference to
    /// it.
    pub fn find<I: Into<ContainerId>>(&self, id: I) -> WmResult<&Container> {
        self.containers.find(id)
    }

    /// Attempt to find a `Container` given its `ContainerId`, returning a mutable reference to
    /// it.
    pub fn find_mut<I: Into<ContainerId>>(&mut self, id: I) -> WmResult<&mut Container> {
        self.containers.find_mut(id)
    }

    /// Returns an iterator over the `ContainerList`.
    #[allow(unused)]
    pub fn iter_containers(&self) -> WmResult<std::collections::vec_deque::Iter<Container>> {
        Ok(self.containers.iter())
    }

    /// Take an already instantiated `Container`, inserting it into the `ContainerList`.
    ///
    /// A new `ContainerId` is generated for the container. This is used for moving `Container`s
    /// between workspaces.
    pub fn insert_container(&mut self, container: Container) -> WmResult<ContainerId> {
        self.containers.container_insert_back(container)
    }

    pub fn screen(&self) -> Geometry {
        self.screen_size
    }

    pub fn swap<I: Into<ContainerId>>(&mut self, a: I, b: I) -> WmResult {
        self.containers.swap(a, b)?;
        Ok(())
    }
}

pub type WorkspaceId = u32;

pub type Workspaces = Vec<Workspace>;
