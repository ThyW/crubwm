#![allow(dead_code)]
use crate::errors::WmResult;

use super::geometry::Geometry;
use std::{cell::RefCell, rc::Rc};

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
}

type Link = Option<Rc<RefCell<ContainerListNode>>>;

struct ContainerListNode {
    data: Container,
    id: (u32, u32),
    next: Link,
    prev: Link,
}

impl ContainerListNode {
    fn new(cont: Container, workspace_id: u32, id: u32) -> Self {
        Self {
            data: cont,
            id: (workspace_id, id),
            next: None,
            prev: None,
        }
    }
}

/// A container list a doubly linked list which is responsible for managing  windows.
/// It consists of reference counted pointers to ConainerNodes.
/// It supports basic operations of addition, removal, length information and an emptiness check.
pub struct ContainerList {
    last_id: u32,
    ws_id: u32,
    first: Link,
    last: Link,
    len: u32,
}

impl ContainerList {
    /// Construct a new container list. This is only done on workspace initialization.
    pub fn new(ws_id: u32) -> Self {
        Self {
            last_id: 0,
            first: None,
            last: None,
            len: 0,
            ws_id,
        }
    }

    fn new_id(&mut self) -> u32 {
        self.last_id += 1;
        self.last_id
    }

    /// Add a new container from the front of the list.
    ///
    /// This function returns the unique ID of the new node.
    pub fn add_front(&mut self, c: Client) -> WmResult<u32> {
        let cont = Container::new(c);
        let id = self.new_id();
        let mut node = ContainerListNode::new(cont, self.ws_id, id);

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
    pub fn add_back(&mut self, c: Client) -> WmResult<u32> {
        let cont = Container::new(c);
        let id = self.new_id();
        let mut node = ContainerListNode::new(cont, self.ws_id, id);

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

    fn find(&self, id: u32) -> WmResult<Rc<RefCell<ContainerListNode>>> {
        if self.is_empty() {
            return Err("container list error: container list is empty.".into());
        }

        for node in self.first.as_ref() {
            let n = node.try_borrow()?;
            if n.id.1 == id {
                return Ok(node.clone());
            }
        }
        Err("container list error: given id not found.".into())
    }

    /// Swap two nodes, given their IDs.
    ///
    /// A nodes ID can be obtained from the `id_for_wid()` and `id_for_pid()` methods.
    pub fn swap(&mut self, a: u32, b: u32) -> WmResult {
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
    pub fn id_for_wid(&self, wid: u32) -> WmResult<u32> {
        for node in self.first.iter() {
            let n = node.try_borrow()?;
            match n.data {
                Container::InLayout(c) => {
                    if c.wid == wid {
                        return Ok(n.id.1);
                    }
                }
                Container::Floating(c) => {
                    if c.wid == wid {
                        return Ok(n.id.1);
                    }
                }
                _ => (),
            }
        }
        Err("container list error: id not found.".into())
    }

    /// Return the node's ID given the client's pid.
    pub fn id_for_pid(&self, pid: u32) -> WmResult<u32> {
        for node in self.first.iter() {
            let n = node.try_borrow()?;
            match n.data {
                Container::InLayout(c) => {
                    if c.pid == pid {
                        return Ok(n.id.1);
                    }
                }
                Container::Floating(c) => {
                    if c.pid == pid {
                        return Ok(n.id.1);
                    }
                }
                _ => (),
            }
        }
        Err("container list error: id not found.".into())
    }
}

fn new_rc(n: ContainerListNode) -> Rc<RefCell<ContainerListNode>> {
    Rc::new(RefCell::new(n))
}

struct IterCursor {
    curr: Option<Rc<RefCell<ContainerListNode>>>,
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
