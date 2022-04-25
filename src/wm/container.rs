#![allow(dead_code)]
use std::collections::VecDeque;

use crate::errors::WmResult;

pub const CT_TILING: u8 = 1 << 0;
pub const CT_FLOATING: u8 = 1 << 1;
pub const CT_EMPTY: u8 = 1 << 2;

use super::geometry::Geometry;

/// Unique identifier for a client.
pub type ClientId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Unique identifier for containers.
pub struct ContainerId {
    workspace_id: u32,
    container_id: u32,
}

impl ContainerId {
    pub fn new<I: Into<u32>>(ws: I, new_id: I) -> Self {
        Self {
            workspace_id: ws.into(),
            container_id: new_id.into(),
        }
    }

    pub fn workspace(&self) -> u32 {
        self.workspace_id
    }

    pub fn container(&self) -> u32 {
        self.container_id
    }
}

impl From<(u32, u32)> for ContainerId {
    fn from(i: (u32, u32)) -> Self {
        Self::new(i.0, i.1)
    }
}

impl std::fmt::Display for ContainerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "w: {}; c: {}", self.workspace_id, self.container_id)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Client {
    wid: u32,
    pid: u32,
    pub geometry: Geometry,
    cid: ClientId,
}

impl Client {
    pub fn new(wid: u32, pid: u32, geometry: Geometry, cid: ClientId) -> Self {
        Self {
            wid,
            pid,
            geometry,
            cid,
        }
    }

    pub fn no_pid(wid: u32, geometry: Geometry, cid: ClientId) -> Self {
        Self {
            wid,
            pid: 0,
            geometry,
            cid,
        }
    }

    pub fn wid(&self) -> u32 {
        self.wid
    }
    pub fn pid(&self) -> u32 {
        self.pid
    }
    pub fn geometry(&self) -> Geometry {
        self.geometry
    }
}

#[derive(Debug, Clone)]
pub enum ContainerType {
    Empty(Geometry),
    InLayout(Client),
    Floating(Client),
}

#[derive(Debug, Clone)]
pub struct Container {
    container_type: ContainerType,
    id: ContainerId,
}

impl Container {
    pub fn new(c: Client, id: ContainerId, contype: u8) -> Self {
        let container_type = match contype {
            CT_FLOATING => ContainerType::new(c).into_floating().unwrap(),
            _ => ContainerType::new(c),
        };

        Self { container_type, id }
    }

    pub fn data(&self) -> &ContainerType {
        &self.container_type
    }

    pub fn data_mut(&mut self) -> &mut ContainerType {
        &mut self.container_type
    }
}

impl Default for ContainerType {
    fn default() -> Self {
        Self::Empty(Geometry::default())
    }
}

impl ContainerType {
    /// Create a new, in-layout container.
    pub fn new(c: Client) -> Self {
        Self::InLayout(c)
    }

    /// Turn an in-layout container to a floating container.
    pub fn into_floating(self) -> Option<Self> {
        match self {
            Self::InLayout(c) => Some(Self::Floating(c)),
            Self::Floating(_) => Some(self),
            Self::Empty(_) => None,
        }
    }

    /// Turn a floating container into an in-layout one.
    pub fn into_layout(self) -> Option<Self> {
        match self {
            Self::InLayout(_) => Some(self),
            Self::Floating(c) => Some(Self::InLayout(c)),
            Self::Empty(_) => None,
        }
    }

    /// If the container is not empty, return the Client of this container and make the container
    /// empty.
    pub fn take(&mut self) -> Option<Client> {
        let c = match self {
            Self::InLayout(c) => Some(*c),
            Self::Floating(c) => Some(*c),
            Self::Empty(_) => None,
        };

        if let Some(client) = c {
            let g = client.geometry;
            let _ = std::mem::replace(self, Self::Empty(g));
            return c;
        }

        None
    }

    /// Get the client geometry.
    pub fn geometry(&self) -> Geometry {
        match self {
            Self::Empty(g) => *g,
            Self::InLayout(c) => c.geometry,
            Self::Floating(c) => c.geometry,
        }
    }

    /// Return client window id, if the container is empty, retrun `None`.
    pub fn wid(&self) -> Option<u32> {
        match self {
            Self::Empty(_) => None,
            Self::InLayout(c) => Some(c.wid),
            Self::Floating(c) => Some(c.wid),
        }
    }

    /// Return client process id, if the container is empty, return `None`.
    pub fn pid(&self) -> Option<u32> {
        match self {
            Self::Empty(_) => None,
            Self::InLayout(c) => Some(c.pid),
            Self::Floating(c) => Some(c.pid),
        }
    }
}

#[derive(Debug)]
pub struct ContainerList {
    containers: VecDeque<Container>,
    workspace_id: u32,
    last_container_id: u32,
}

impl ContainerList {
    pub fn new(workspace_id: u32) -> Self {
        Self {
            containers: VecDeque::new(),
            workspace_id,
            last_container_id: 0,
        }
    }

    fn new_id(&mut self) -> ContainerId {
        self.last_container_id += 1;
        ContainerId::new(self.workspace_id, self.last_container_id)
    }

    fn inner_find(&self, id: ContainerId) -> Option<usize> {
        if !id.workspace() == self.workspace_id {
            None
        } else {
            if let Some((index, _)) = self.containers.iter().enumerate().find(|(_, c)| c.id == id) {
                return Some(index);
            }

            None
        }
    }

    pub fn insert_front(&mut self, c: Client, cont_type: u8) -> ContainerId {
        let id = self.new_id();
        let cont = Container::new(c, id, cont_type);
        self.containers.push_front(cont);

        id
    }

    pub fn insert_back(&mut self, c: Client, cont_type: u8) -> ContainerId {
        let id = self.new_id();
        let cont = Container::new(c, id, cont_type);
        self.containers.push_back(cont);

        id
    }

    pub fn swap(&mut self, a: ContainerId, b: ContainerId) -> WmResult {
        if let Some(a) = self.inner_find(a) {
            if let Some(b) = self.inner_find(b) {
                self.containers.swap(a, b);
                return Ok(());
            };
            return Err(format!("container list error: wrong container id -> {b}").into());
        }
        return Err(format!("container list error: wrong container id -> {a}").into());
    }

    pub fn remove(&mut self, cid: ContainerId) -> WmResult<Container> {
        if let Some(i) = self.inner_find(cid) {
            if let Some(c) = self.containers.remove(i) {
                return Ok(c);
            }
            return Err(format!("container list error: unable to find {cid}").into());
        }
        return Err(format!("container list error: unable to remove {cid}").into());
    }

    pub fn get_all_mut(&mut self) -> std::collections::vec_deque::IterMut<Container> {
        self.containers.iter_mut()
    }

    pub fn get_all(&self) -> std::collections::vec_deque::Iter<Container> {
        self.containers.iter()
    }

    pub fn find_mut(&mut self, cid: ContainerId) -> WmResult<&mut Container> {
        if let Some(i) = self.inner_find(cid) {
            return Ok(&mut self.containers[i]);
        }
        return Err(format!("container list error: unable to find {cid}").into());
    }

    pub fn find(&self, cid: ContainerId) -> WmResult<&Container> {
        if let Some(i) = self.inner_find(cid) {
            return Ok(&self.containers[i]);
        }
        return Err(format!("container list error: unable to find {cid}").into());
    }

    pub fn id_for_wid(&self, wid: u32) -> WmResult<ContainerId> {
        for c in &self.containers {
            if let Some(cwid) = c.container_type.wid() {
                if wid == cwid {
                    return Ok(c.id);
                }
            }
        }

        return Err(format!(
            "container list node: unable to find a container for window id: {wid}"
        )
        .into());
    }

    pub fn id_for_pid(&self, pid: u32) -> WmResult<ContainerId> {
        for c in &self.containers {
            if let Some(cpid) = c.container_type.pid() {
                if pid == cpid {
                    return Ok(c.id);
                }
            }
        }

        return Err(format!(
            "container list node: unable to find a container for window id: {pid}"
        )
        .into());
    }
}
