#![allow(dead_code)]
use crate::errors::WmResult;

use super::geometry::Geometry;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, PartialEq, Eq)]
/// Used to identify containers.
pub struct ContainerId {
    workspace_id: u32,
    container_id: u32
}

impl ContainerId {
    pub fn new<I: Into<u32>>(ws: I, new_id: I) -> Self {
        Self {
            workspace_id: ws.into(),
            container_id: new_id.into()
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Client {
    pub wid: u32,
    pub pid: u32,
    pub geometry: Geometry,
}

impl Client {
    pub fn new(wid: u32, pid: u32, geometry: Geometry) -> Self {
        Self { wid, pid, geometry }
    }

    pub fn no_pid(wid: u32, geometry: Geometry) -> Self {
        Self {
            wid,
            pid: 0,
            geometry,
        }
    }
}

#[derive(Clone)]
pub enum Container {
    Empty(Geometry),
    InLayout(Client),
    Floating(Client),
}

impl Default for Container {
    fn default() -> Self {
        Self::Empty(Geometry::default())
    }
}

impl Container {
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

    pub fn geometry(&self) -> Geometry {
        match self {
            Self::Empty(g) => g.clone(),
            Self::InLayout(c) => c.geometry.clone(),
            Self::Floating(c) => c.geometry.clone(),
        }
    }

    pub fn wid(&self) -> Option<u32> {
        match self {
            Self::Empty(_) => None,
            Self::InLayout(c) => Some(c.wid),
            Self::Floating(c) => Some(c.wid),
        }
    }
}

type Link = Option<Rc<RefCell<ContainerListNode>>>;

pub(crate) struct ContainerListNode {
    data: Container,
    id: ContainerId,
    next: Link,
    prev: Link,
}

impl ContainerListNode {
    fn new(cont: Container, id: ContainerId) -> Self {
        Self {
            data: cont,
            id,
            next: None,
            prev: None,
        }
    }

    pub fn data(&self) -> &Container {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Container {
        &mut self.data
    }
}

/// A container list a doubly linked list which is responsible for managing  windows.
/// It consists of reference counted pointers to ConainerNodes.
/// It supports basic operations of addition, removal, length information and an emptiness check.
pub struct ContainerList {
    last_id: u32,
    workspace_id: u32,
    first: Link,
    last: Link,
    len: u32,
}

impl ContainerList {
    /// Construct a new container list. This is only done on workspace initialization.
    pub fn new<I: Into<u32>>(workspace: I) -> Self {
        Self {
            last_id: 0,
            workspace_id: workspace.into(),
            first: None,
            last: None,
            len: 0,
        }
    }

    fn new_id(&mut self) -> ContainerId {
        self.last_id += 1;
        return ContainerId::new(self.workspace_id, self.last_id)
    }

    /// Get number of nodes in the list.
    pub fn len(&self) -> u32 {
        self.len
    }

    /// Add a new container from the front of the list.
    ///
    /// This function returns the unique ID of the new node.
    pub fn add_front(&mut self, c: Client) -> WmResult<ContainerId> {
        let cont = Container::new(c);
        let id = self.new_id();
        let mut node = ContainerListNode::new(cont, id);

        // if the 'first node' exists, make the 'new' node 'first node' and add the 'new first node' as the
        // 'old first node's' previous.
        if let Some(first_node) = &self.first {
            node.next = Some(first_node.clone());
            let node_rc = new_rc(node);
            first_node.try_borrow_mut()?.prev = Some(node_rc);
        } else {
            let node_rc = new_rc(node);
            self.first = Some(node_rc.clone());
            self.last = Some(node_rc.clone())
        }

        self.len += 1;

        Ok(id)
    }

    /// Add a new container to the back of the list.
    ///
    /// This fucntion returns the unique ID of the new node.
    pub fn add_back(&mut self, c: Client) -> WmResult<ContainerId> {
        let cont = Container::new(c);
        let id = self.new_id();
        let mut node = ContainerListNode::new(cont, id);

        if let Some(last_node) = &self.last {
            node.prev = Some(last_node.clone());
            let node_rc = new_rc(node);
            last_node.try_borrow_mut()?.next = Some(node_rc)
        } else {
            let node_rc = new_rc(node);
            self.first = Some(node_rc.clone());
            self.last = Some(node_rc.clone())
        }

        self.len += 1;

        Ok(id)
    }

    /// Checks whether the nodes is empty or not.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn find<I: Into<ContainerId> + Copy>(&self, id: I) -> WmResult<Rc<RefCell<ContainerListNode>>> {
        if self.is_empty() {
            return Err("container list error: container list is empty.".into());
        }

        for node in self.first.as_ref() {
            let n = node.try_borrow()?;
            if n.id == id.into() {
                return Ok(node.clone());
            }
        }
        Err("container list error: given id not found.".into())
    }

    /// Swap two nodes, given their IDs.
    ///
    /// A nodes ID can be obtained from the `id_for_wid()` and `id_for_pid()` methods.
    pub fn swap<I: Into<ContainerId> + Copy>(&mut self, a: I, b: I) -> WmResult {
        // get the two nodes
        let na = self.find(a)?;
        let nb = self.find(b)?;

        // swap the data and the id, while keeping neighbors
        // first, try borrowing both nodes.
        let mut borrowed_a = na.try_borrow_mut()?;
        let mut borrowed_b = nb.try_borrow_mut()?;
        // assign the data from the first node into a temporary tuple.
        let temp_a = (borrowed_a.data.clone(), borrowed_a.id);

        // perform the swap.
        borrowed_a.data = borrowed_b.data.clone();
        borrowed_a.id = borrowed_b.id;

        borrowed_b.data = temp_a.0;
        borrowed_b.id = temp_a.1;

        Ok(())
    }

    /// Return the node's ID given the client's wid.
    pub fn id_for_wid(&self, wid: u32) -> WmResult<ContainerId> {
        for node in self.first.iter() {
            let n = node.try_borrow()?;
            match n.data {
                Container::InLayout(c) => {
                    if c.wid == wid {
                        return Ok(n.id);
                    }
                }
                Container::Floating(c) => {
                    if c.wid == wid {
                        return Ok(n.id);
                    }
                }
                _ => (),
            }
        }
        Err("container list error: id not found.".into())
    }

    /// Return the node's ID given the client's pid.
    pub fn id_for_pid(&self, pid: u32) -> WmResult<ContainerId> {
        for node in self.first.iter() {
            let n = node.try_borrow()?;
            match n.data {
                Container::InLayout(c) => {
                    if c.pid == pid {
                        return Ok(n.id);
                    }
                }
                Container::Floating(c) => {
                    if c.pid == pid {
                        return Ok(n.id);
                    }
                }
                _ => (),
            }
        }
        Err("container list error: id not found.".into())
    }

    pub(crate) fn get_all(&self) -> Vec<Rc<RefCell<ContainerListNode>>> {
        let mut out = vec![];

        for each in self.first.as_ref().into_iter() {
            out.push(each.clone())
        }

        out
    }

    pub fn remove<I: Into<ContainerId> + Copy>(&mut self, id: I) -> WmResult {
        if self.is_empty() {
            return Err("conatiner list error: contaier list is empty.".into())
        }
        let node_rc = self.find(id)?;

        let mut node = node_rc.try_borrow_mut()?;

        // if this node has no ancestors or predecessors
        if node.next.is_none() && node.prev.is_none() {
            self.first = None;
            self.last = None;
        }

        if node.next.is_some() && node.prev.is_none() {
            let next = node.next.as_ref().unwrap().clone();

            let mut next = next.try_borrow_mut()?;
            next.prev = None;

            node.next = None;
            node.prev = None;
        }
        
        if node.prev.is_some() && node.next.is_none() {
            let prev = node.prev.as_ref().unwrap().clone();

            let mut prev = prev.try_borrow_mut()?;

            prev.next = None;

            node.next = None;
            node.prev = None;
        }

        if node.next.is_some() && node.prev.is_some() {
            let prev = node.prev.as_ref().unwrap().clone();
            let next = node.next.as_ref().unwrap().clone();

            let mut prev = prev.try_borrow_mut()?;
            let mut next = next.try_borrow_mut()?;

            prev.next = node.next.clone();
            next.prev = node.prev.clone();
        }

        Ok(())
    }
}

fn new_rc(n: ContainerListNode) -> Rc<RefCell<ContainerListNode>> {
    Rc::new(RefCell::new(n))
}

pub(crate) struct IterCursor {
    curr: Link,
}

impl Iterator for IterCursor {
    type Item = Rc<RefCell<ContainerListNode>>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.curr.as_ref().unwrap().try_borrow() {
            Ok(i) => {
                if let Some(next) = &i.next {
                    return Some(next.clone());
                } else {
                    return None;
                }
            }
            Err(_) => return None,
        };
    }
}

impl IntoIterator for ContainerListNode {
    type Item = Rc<RefCell<ContainerListNode>>;
    type IntoIter = IterCursor;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { curr: self.next }
    }
}
