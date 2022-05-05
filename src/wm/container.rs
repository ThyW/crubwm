#![allow(dead_code)]
use std::collections::VecDeque;

use crate::{errors::WmResult, wm::geometry::Geometry};

pub const CT_MASK_TILING: u8 = 1 << 0;
pub const CT_MASK_FLOATING: u8 = 1 << 1;
pub const CT_MASK_EMPTY: u8 = 1 << 2;

/// Unique identifier for a client.
pub type ClientId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Unique identifier for containers.
pub struct ContainerId {
    workspace_id: u32,
    container_id: u32,
}

impl ContainerId {
    pub fn new<I: Into<u32>>(workspace_id: I, container_id: I) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            container_id: container_id.into(),
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
    window_id: u32,
    process_id: u32,
    pub geometry: Geometry,
    client_id: ClientId,
}

impl Client {
    pub fn new<I: Into<u32>, C: Into<ClientId>, G: Into<Geometry>>(
        window_id: I,
        process_id: I,
        geometry: G,
        client_id: C,
    ) -> Self {
        Self {
            window_id: window_id.into(),
            process_id: process_id.into(),
            geometry: geometry.into(),
            client_id: client_id.into(),
        }
    }

    pub fn new_without_process_id<I: Into<u32>, C: Into<ClientId>, G: Into<Geometry>>(
        window_id: I,
        geometry: G,
        client_id: C,
    ) -> Self {
        Self {
            window_id: window_id.into(),
            process_id: 0,
            geometry: geometry.into(),
            client_id: client_id.into(),
        }
    }

    pub fn window_id(&self) -> u32 {
        self.window_id
    }
    pub fn process_id(&self) -> Option<u32> {
        if self.process_id == 0 {
            return None;
        }
        Some(self.process_id)
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
    container_id: ContainerId,
}

impl Container {
    pub fn new<C: Into<Client>, I: Into<ContainerId>, T: Into<u8>>(
        client: C,
        id: I,
        container_type_mask: T,
    ) -> Self {
        let container_type = match container_type_mask.into() {
            CT_MASK_FLOATING => ContainerType::new(client.into()).into_floating().unwrap(),
            _ => ContainerType::new(client.into()),
        };

        Self {
            container_type,
            container_id: id.into(),
        }
    }

    pub fn data(&self) -> &ContainerType {
        &self.container_type
    }

    pub fn data_mut(&mut self) -> &mut ContainerType {
        &mut self.container_type
    }

    pub fn id(&self) -> &ContainerId {
        &self.container_id
    }
}

impl Default for ContainerType {
    fn default() -> Self {
        Self::Empty(Geometry::default())
    }
}

impl ContainerType {
    /// Create a new, in-layout container.
    fn new(c: Client) -> Self {
        Self::InLayout(c)
    }

    /// Turn an in-layout container to a floating container.
    fn into_floating(self) -> Option<Self> {
        match self {
            Self::InLayout(c) => Some(Self::Floating(c)),
            Self::Floating(_) => Some(self),
            Self::Empty(_) => None,
        }
    }

    /// Turn a floating container into an in-layout one.
    fn into_layout(self) -> Option<Self> {
        match self {
            Self::InLayout(_) => Some(self),
            Self::Floating(c) => Some(Self::InLayout(c)),
            Self::Empty(_) => None,
        }
    }

    /// If the container is not empty, return the Client of this container and make the container
    /// empty.
    fn take(&mut self) -> Option<Client> {
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
    pub fn window_id(&self) -> Option<u32> {
        match self {
            Self::Empty(_) => None,
            Self::InLayout(c) => Some(c.window_id),
            Self::Floating(c) => Some(c.window_id),
        }
    }

    /// Return client process id, if the container is empty, return `None`.
    pub fn process_id(&self) -> Option<u32> {
        match self {
            Self::Empty(_) => None,
            Self::InLayout(c) => {
                if c.process_id == 0 {
                    return None;
                }
                Some(c.process_id)
            }
            Self::Floating(c) => {
                if c.process_id == 0 {
                    return None;
                }
                Some(c.process_id)
            }
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
            if let Some((index, _)) = self
                .containers
                .iter()
                .enumerate()
                .find(|(_, c)| c.container_id == id)
            {
                return Some(index);
            }

            None
        }
    }

    pub fn insert_front<C: Into<Client>, I: Into<u8>>(
        &mut self,
        client: C,
        container_type_mask: I,
    ) -> ContainerId {
        let id = self.new_id();
        let cont = Container::new(client, id, container_type_mask);
        self.containers.push_front(cont);

        id
    }

    pub fn insert_back<C: Into<Client>, I: Into<u8>>(
        &mut self,
        client: C,
        container_type_mask: I,
    ) -> ContainerId {
        let id = self.new_id();
        let cont = Container::new(client, id, container_type_mask);
        self.containers.push_back(cont);

        id
    }

    pub fn swap<I: Into<ContainerId>>(&mut self, a: I, b: I) -> WmResult {
        let a = a.into();
        let b = b.into();
        if let Some(a) = self.inner_find(a) {
            if let Some(b) = self.inner_find(b) {
                self.containers.swap(a, b);
                return Ok(());
            };
            return Err(format!("container list error: wrong container id -> {b}").into());
        }
        return Err(format!("container list error: wrong container id -> {a}").into());
    }

    pub fn remove<C: Into<ContainerId>>(&mut self, container_id: C) -> WmResult<Container> {
        let c = container_id.into();
        if let Some(i) = self.inner_find(c) {
            if let Some(c) = self.containers.remove(i) {
                return Ok(c);
            }
            return Err(format!("container list error: unable to find {c}").into());
        }
        return Err(format!("container list error: unable to remove {c}").into());
    }

    pub fn iter_mut(&mut self) -> std::collections::vec_deque::IterMut<Container> {
        self.containers.iter_mut()
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<Container> {
        self.containers.iter()
    }

    pub fn find_mut<C: Into<ContainerId>>(&mut self, container_id: C) -> WmResult<&mut Container> {
        let c = container_id.into();
        if let Some(i) = self.inner_find(c) {
            return Ok(&mut self.containers[i]);
        }
        return Err(format!("container list error: unable to find {}", c).into());
    }

    pub fn find<C: Into<ContainerId>>(&self, container_id: C) -> WmResult<&Container> {
        let c = container_id.into();
        if let Some(i) = self.inner_find(c) {
            return Ok(&self.containers[i]);
        }
        return Err(format!("container list error: unable to find {c}").into());
    }

    pub fn id_for_window<I: Into<u32>>(&self, window_id: I) -> WmResult<ContainerId> {
        let wid = window_id.into();
        for c in &self.containers {
            if let Some(cwid) = c.container_type.window_id() {
                if wid == cwid {
                    return Ok(c.container_id);
                }
            }
        }

        return Err(format!(
            "container list node: unable to find a container for window id: {wid}"
        )
        .into());
    }

    pub fn id_for_process<I: Into<u32>>(&self, process_id: I) -> WmResult<ContainerId> {
        let pid = process_id.into();
        for c in &self.containers {
            if let Some(cpid) = c.container_type.process_id() {
                if pid == cpid {
                    return Ok(c.container_id);
                }
            }
        }

        return Err(format!(
            "container list node: unable to find a container for window id: {pid}"
        )
        .into());
    }

    pub fn next_for_id<C: Into<ContainerId>>(&self, id: C) -> WmResult<&Container> {
        if let Some(mut index) = self.inner_find(id.into()) {
            if index == self.containers.len() - 1 {
                index = 0;
            }
            if let Some(cont) = self.containers.get(index + 1) {
                return Ok(cont);
            }
        }

        Err("container list error: unable to get next container!".into())
    }

    pub fn previous_for_id<C: Into<ContainerId>>(&self, id: C) -> WmResult<&Container> {
        if let Some(mut index) = self.inner_find(id.into()) {
            if index == 0 {
                index = self.containers.len() - 1;
            }
            if let Some(cont) = self.containers.get(index - 1) {
                return Ok(cont);
            }
        }

        Err("container list error: unable to get next container!".into())
    }

    pub fn insert_back_full(&mut self, mut container: Container) -> WmResult<ContainerId> {
        let new_id = self.new_id();
        container.container_id = new_id;
        self.containers.push_back(container);
        Ok(new_id)
    }
}
