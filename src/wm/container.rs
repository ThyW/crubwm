#![allow(dead_code)]
use std::{collections::VecDeque, sync::Arc};

use x11rb::protocol::xproto::{ChangeWindowAttributesAux, ConfigureWindowAux, ConnectionExt};

use crate::{
    config::Config,
    errors::{Error, WmResult},
    wm::geometry::{ClientAttributes, Geometry},
};

pub struct ContainerTypeMask(u8);

impl ContainerTypeMask {
    pub const TILING: u8 = 1 << 0;
    pub const FLOATING: u8 = 1 << 1;
    const CT_MASK_EMPTY: u8 = 1 << 2;

    pub fn try_from(c: String) -> WmResult<u8> {
        let s = c.to_lowercase();

        match &s[..] {
            "in_layout" => Ok(Self::TILING),
            "float" => Ok(Self::FLOATING),
            _ => Err(format!("{c} is not a valid layout type string").into()),
        }
    }
}

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
    // TODO: Should have a properties cache.
    window_id: u32,
    process_id: u32,
    pub geometry: Geometry,
    pub attributes: ClientAttributes,
    client_id: ClientId,
}

impl Client {
    pub fn new<I: Into<u32>, CID: Into<ClientId>, G: Into<Geometry>>(
        window_id: I,
        process_id: I,
        geometry: G,
        client_id: CID,
        config: &Config,
    ) -> Self {
        let attrs = ClientAttributes::from(config.clone());
        Self {
            window_id: window_id.into(),
            process_id: process_id.into(),
            geometry: geometry.into(),
            client_id: client_id.into(),
            attributes: attrs,
        }
    }

    pub fn new_without_process_id<I: Into<u32>, C: Into<ClientId>, G: Into<Geometry>>(
        window_id: I,
        geometry: G,
        client_id: C,
        config: &Config,
    ) -> Self {
        let attributes = ClientAttributes::from(config.clone());
        Self {
            window_id: window_id.into(),
            process_id: 0,
            geometry: geometry.into(),
            client_id: client_id.into(),
            attributes,
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

    pub fn with_gaps(&self) -> Geometry {
        let mut geom = self.geometry();
        geom.x += self.attributes.gap_left as i16;
        geom.y += self.attributes.gap_top as i16;
        geom.width -= 2 * self.attributes.gap_right as u16;
        geom.height -= 2 * self.attributes.gap_bottom as u16;

        geom
    }
    pub fn with_gaps_inner(&self) -> Geometry {
        let mut geom = self.geometry();
        geom.x += self.attributes.gap_left as i16 / 2;
        geom.y += self.attributes.gap_top as i16 / 2;
        geom.width -= self.attributes.gap_right as u16;
        geom.height -= self.attributes.gap_bottom as u16;

        geom
    }

    pub fn with_borders(&self) -> (Geometry, u32, u16, u16, u16) {
        let mut geom = self.with_gaps();
        geom.width -= 2 * self.attributes.border_size as u16;
        geom.height -= 2 * self.attributes.border_size as u16;
        let bytes = self.attributes.border_color.to_le_bytes();

        (
            geom,
            self.attributes.border_size,
            (bytes[0] as u16) << 8,
            (bytes[1] as u16) << 8,
            (bytes[2] as u16) << 8,
        )
    }

    pub fn border_color(&self) -> (u16, u16, u16) {
        let bytes = self.attributes.border_color.to_le_bytes();

        (
            (bytes[2] as u16) << 8 | (bytes[2] as u16),
            (bytes[1] as u16) << 8 | (bytes[1] as u16),
            (bytes[0] as u16) << 8 | (bytes[0] as u16),
        )
    }

    pub fn border_width(&self) -> u32 {
        self.attributes.border_size
    }

    pub fn change_config(&mut self, config: &Config) {
        self.attributes = ClientAttributes::from(config.clone())
    }

    pub fn draw_borders<C: x11rb::connection::Connection>(
        &self,
        connection: Arc<C>,
        default_colormap: u32,
    ) -> WmResult {
        connection.configure_window(self.window_id(), &self.with_borders().0.into())?;

        let border_colors = self.border_color();
        let border_size = self.border_width();
        let pixel = connection
            .alloc_color(
                default_colormap,
                border_colors.0,
                border_colors.1,
                border_colors.2,
            )?
            .reply()?
            .pixel;
        connection.change_window_attributes(
            self.window_id(),
            &ChangeWindowAttributesAux::new().border_pixel(pixel),
        )?;
        connection.configure_window(
            self.window_id(),
            &ConfigureWindowAux::new().border_width(Some(border_size)),
        )?;
        connection.free_colors(default_colormap, 0, &[pixel])?;

        Ok(())
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
    last_position: Option<(i32, i32)>,
}

impl Container {
    pub fn new<C: Into<Client>, I: Into<ContainerId>, T: Into<u8>>(
        client: C,
        id: I,
        container_type_mask: T,
    ) -> Self {
        let container_type = match container_type_mask.into() {
            ContainerTypeMask::FLOATING => {
                ContainerType::new(client.into()).into_floating().unwrap()
            }
            _ => ContainerType::new(client.into()),
        };

        Self {
            container_type,
            container_id: id.into(),
            last_position: None,
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

    pub fn change_to_layout(&mut self) -> WmResult {
        self.container_type =
            self.data_mut().clone().into_layout().ok_or_else(|| {
                Error::Generic("unable to change container type to InLayout".into())
            })?;

        Ok(())
    }

    pub fn change_to_floating(&mut self) -> WmResult {
        self.container_type =
            self.data_mut().clone().into_floating().ok_or_else(|| {
                Error::Generic("unable to change contdainer type to Floating".into())
            })?;

        Ok(())
    }

    pub fn is_floating(&self) -> bool {
        if matches!(self.container_type, ContainerType::Floating(_)) {
            return true;
        }
        false
    }

    pub fn is_in_layout(&self) -> bool {
        if matches!(self.container_type, ContainerType::InLayout(_)) {
            return true;
        }
        false
    }

    pub fn last_position(&self) -> Option<(i32, i32)> {
        self.last_position
    }

    pub fn change_last_position<I: Into<i32>>(&mut self, new_position: (I, I)) {
        self.last_position = Some((new_position.0.into(), new_position.1.into()));
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

    /// Set the client geometry.
    pub fn set_geometry(&mut self, geom: Geometry) {
        match self {
            Self::Empty(g) => *g = geom,
            Self::InLayout(c) => c.geometry = geom,
            Self::Floating(c) => c.geometry = geom,
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

#[derive(Debug, Clone)]
pub struct ContainerList {
    containers: VecDeque<Container>,
    workspace_id: u32,
    last_container_id: u32,
}

impl ContainerList {
    /// Create a new container list.
    pub fn new(workspace_id: u32) -> Self {
        Self {
            containers: VecDeque::new(),
            workspace_id,
            last_container_id: 0,
        }
    }

    /// Generate a new container id.
    fn new_id(&mut self) -> ContainerId {
        self.last_container_id += 1;
        ContainerId::new(self.workspace_id, self.last_container_id)
    }

    /// Given a container id, return the index of the container in the container list.
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

    /// Given a client and a container type mask, create a new container and insert it into the
    /// front of the container list.
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

    /// Given a Client and a container type mask, create a new container and insert it into the
    /// back of the container list.
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

    /// Given to `ContainerId`s, first validate them and them swap the `Container`s in place.
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
        Err(format!("container list error: wrong container id -> {a}").into())
    }

    /// Given a `ContainerId`, remove it from the container list, returning the client.
    pub fn remove<C: Into<ContainerId>>(&mut self, container_id: C) -> WmResult<Container> {
        let c = container_id.into();
        if let Some(i) = self.inner_find(c) {
            if let Some(c) = self.containers.remove(i) {
                return Ok(c);
            }
            return Err(format!("container list error: unable to find {c}").into());
        }
        Err(format!("container list error: unable to remove {c}").into())
    }

    /// Mutably iterate over the `Container`s in the container list.
    pub fn iter_mut(&mut self) -> std::collections::vec_deque::IterMut<Container> {
        self.containers.iter_mut()
    }

    /// Immutably iterate over the `Container`s in the container list.
    pub fn iter(&self) -> std::collections::vec_deque::Iter<Container> {
        self.containers.iter()
    }

    /// Mutably iterate over the `Container`s in the container list, while also returning the
    /// number of `Container`s that are of the type of `InLayout`.
    pub fn iter_in_layout_mut(
        &mut self,
    ) -> (usize, std::collections::vec_deque::IterMut<Container>) {
        let len = self.containers.iter().filter(|x| x.is_in_layout()).count();
        (len, self.containers.iter_mut())
    }

    /// Given a `ContainerId`, return a result containing a mutable reference to that `Container`.
    pub fn find_mut<C: Into<ContainerId>>(&mut self, container_id: C) -> WmResult<&mut Container> {
        let c = container_id.into();
        if let Some(i) = self.inner_find(c) {
            return Ok(&mut self.containers[i]);
        }
        Err(format!("container list error: unable to find {}", c).into())
    }

    /// Given a `ContainerId`, return a result containing an immutable reference to that `Container`.
    pub fn find<C: Into<ContainerId>>(&self, container_id: C) -> WmResult<&Container> {
        let c = container_id.into();
        if let Some(i) = self.inner_find(c) {
            return Ok(&self.containers[i]);
        }
        Err(format!("container list error: unable to find {c}").into())
    }

    /// Given an X window id(u32), return the `ContainerId` of the `Container`, which holds the client
    /// with the specified window id.
    pub fn id_for_window<I: Into<u32>>(&self, window_id: I) -> WmResult<ContainerId> {
        let wid = window_id.into();
        for c in &self.containers {
            if let Some(cwid) = c.container_type.window_id() {
                if wid == cwid {
                    return Ok(c.container_id);
                }
            }
        }

        Err(format!("container list node: unable to find a container for window id: {wid}").into())
    }

    /// Given a process id, return the `ContainerId` of the `Container`, which holds the client
    /// with the specified process id.
    pub fn id_for_process<I: Into<u32>>(&self, process_id: I) -> WmResult<ContainerId> {
        let pid = process_id.into();
        for c in &self.containers {
            if let Some(cpid) = c.container_type.process_id() {
                if pid == cpid {
                    return Ok(c.container_id);
                }
            }
        }

        Err(format!("container list node: unable to find a container for window id: {pid}").into())
    }

    /// Return an immutable reference to the next `Container` in the list, given a `ContainerId`.
    pub fn next_for_id<C: Into<ContainerId>>(&self, id: C) -> WmResult<&Container> {
        if let Some(mut index) = self.inner_find(id.into()) {
            if index == self.containers.len() - 1 {
                index = 0;
            } else {
                index += 1
            }
            if let Some(cont) = self.containers.get(index) {
                return Ok(cont);
            }
        }

        Err("container list error: unable to get next container!".into())
    }

    /// Return an immutable reference to the previous `Container` in the list, given a `ContainerId`.
    pub fn previous_for_id<C: Into<ContainerId>>(&self, id: C) -> WmResult<&Container> {
        if let Some(mut index) = self.inner_find(id.into()) {
            if index == 0 {
                index = self.containers.len() - 1;
            } else {
                index -= 1
            }
            if let Some(cont) = self.containers.get(index) {
                return Ok(cont);
            }
        }

        Err("container list error: unable to get next container!".into())
    }

    /// Given an already existing `Container`, generate a new id for it and insert it into the back
    /// of the list, returning the new `ContainerId`.
    pub fn container_insert_back(&mut self, mut container: Container) -> WmResult<ContainerId> {
        let new_id = self.new_id();
        container.container_id = new_id;
        self.containers.push_back(container);
        Ok(new_id)
    }
}
