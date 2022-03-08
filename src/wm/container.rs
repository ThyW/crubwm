#![allow(dead_code)]
use std::rc::Rc;

use super::geometry::Geometry;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Client {
    wid: u32,
    pid: u32,
    geometry: Geometry,
}

impl Client {
    pub fn new(wid: u32, pid: u32, geometry: Geometry) -> Self {
        Self { wid, pid, geometry }
    }

    pub fn wid(&self) -> u32 {
        self.wid
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }
}

#[derive(Clone)]
pub enum ContainerType {
    Empty,
    InLayout,
    Floating,
}

impl Default for ContainerType {
    fn default() -> Self {
        Self::InLayout
    }
}

#[derive(Clone)]
pub struct ContainerData {
    client: Client,
    container_type: ContainerType,
}

impl ContainerData {
    pub fn new(client: Client, container_type: ContainerType) -> Self {
        Self {
            client,
            container_type,
        }
    }
}

pub struct ContainerNode {
    data: ContainerData,
    next: Option<Rc<ContainerNode>>,
    prev: Option<Rc<ContainerNode>>,
}

impl ContainerNode {
    pub fn new(data: ContainerData) -> Self {
        Self {
            data,
            next: None,
            prev: None,
        }
    }

    fn set_next(&mut self, next: Rc<Self>) {
        self.next = Some(next)
    }

    fn set_prev(&mut self, prev: Rc<Self>) {
        self.prev = Some(prev)
    }
}

/// A container list a doubly linked list which is responsible for managing  windows.
/// It consists of reference counted pointers to ConainerNodes.
/// It supports basic operations of addition, removal, length information, emptiness check.
pub struct ContainerList {
    pub first: Option<Rc<ContainerNode>>,
    pub last: Option<Rc<ContainerNode>>,
    focused: Option<Rc<ContainerNode>>,
    len: usize,
}

impl ContainerList {
    pub const fn new() -> Self {
        Self {
            first: None,
            last: None,
            focused: None,
            len: 0,
        }
    }

    /// Return the number of nodes in the list.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Checks if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Add a new client to the end of the list.
    pub fn append(&mut self, c: Client) {
        let data = ContainerData::new(c, ContainerType::default());

        let node = ContainerNode::new(data);
        let mut rc = Rc::new(node);

        if self.is_empty() {
            self.first = Some(rc.clone());
            self.last = Some(rc.clone())
        } else {
            if let Some(mut last) = self.last.take() {
                if let Some(node) = Rc::get_mut(&mut last) {
                    node.set_next(rc.clone());
                    if let Some(new) = Rc::get_mut(&mut rc) {
                        new.set_prev(last);
                        self.last = Some(rc);
                        self.len += 1
                    }
                }
            }
        }
    }

    /// Add a new client to the start of the cointainer list.
    pub fn prepend(&mut self, c: Client) {
        let data = ContainerData::new(c, ContainerType::default());

        let node = ContainerNode::new(data);
        let mut rc = Rc::new(node);

        if self.is_empty() {
            self.first = Some(rc.clone());
            self.last = Some(rc.clone());
        } else {
            if let Some(mut first_rc) = self.first.take() {
                if let Some(node_first) = Rc::get_mut(&mut first_rc) {
                    node_first.set_prev(rc.clone());
                    if let Some(new) = Rc::get_mut(&mut rc) {
                        new.set_next(first_rc);
                        self.first = Some(rc);
                        self.len += 1
                    }
                }
            }
        }
    }

    /// Get a reference to a client given its window id, if it is in the list.
    pub fn get_client_by_wid(&self, id: u32) -> Option<&Client> {
        if self.is_empty() {
            return None;
        }

        let mut node = &self.first;
        while node.is_some() {
            let inside = node.as_ref().unwrap();
            if id == inside.data.client.wid() {
                return Some(&inside.data.client);
            } else {
                node = &inside.next
            }
        }

        None
    }

    /// Get a reference to a client given its process id, if it is in the list.
    pub fn get_client_by_pid(&self, id: u32) -> Option<&Client> {
        if self.is_empty() {
            return None;
        }
        let mut node = &self.first;

        while node.is_some() {
            let inside = node.as_ref().unwrap();
            if id == inside.data.client.pid {
                return Some(&inside.data.client);
            } else {
                node = &inside.next;
            }
        }

        None
    }

    /// Helper function: given a window id, attempt to return the node which contains the client
    /// with that window id.
    fn get_rc_by_wid(&self, id: u32) -> Option<Rc<ContainerNode>> {
        if self.is_empty() {
            return None;
        }

        let mut node = &self.first;
        while node.is_some() {
            let inside = node.as_ref().unwrap();
            if id == inside.data.client.wid() {
                return node.clone();
            } else {
                node = &inside.next;
            }
        }

        None
    }

    /// Helper function: given a process id, attempt to return the node which contains the client
    /// with that process id.
    fn get_rc_by_pid(&self, id: u32) -> Option<Rc<ContainerNode>> {
        if self.is_empty() {
            return None;
        }

        let mut node = &self.first;
        while node.is_some() {
            let inside = node.as_ref().unwrap();
            if id == inside.data.client.pid() {
                return node.clone();
            } else {
                node = &inside.next;
            }
        }

        None
    }

    /// Retrun the data of the element in the list and remove the element from the list.
    pub fn pop_first(&mut self) -> Option<ContainerData> {
        if self.is_empty() {
            return None
        }

        if let Some(first) = &mut self.first {
            let next = first.next.as_ref();

            if let Some(next_node) = next {
                if let Some(next_inside) = Rc::get_mut(&mut next_node.clone()) {
                    next_inside.prev = None;
                }
            }

            if let Some(first_inside) = Rc::get_mut(first) {
                first_inside.next = None;
                self.len -= 1;
                return Some(first_inside.data.clone())
            }

        }

        None
    }

    /// Return the data in the last node of the list and remove the node from the list.
    pub fn pop_last(&mut self) -> Option<ContainerData> {
        if self.is_empty() {
            return None
        }

        if let Some(last) = &mut self.last {
            let prev = last.prev.as_ref();

            if let Some(prev_node) = prev {
                if let Some(prev_inside) = Rc::get_mut(&mut prev_node.clone()) {
                    prev_inside.next = None;
                }
            }

            if let Some(last_inside) = Rc::get_mut(last) {
                last_inside.prev = None;
                self.len -= 1;
                return Some(last_inside.data.clone())
            }
        }

        None
    }

    /// Remove a client given its window id.
    pub fn remove_by_wid(&mut self, id: u32) {
        if let Some(mut node) = self.get_rc_by_wid(id) {
            // next and previous node of the 'to be deleted' node.
            let next = node.next.as_ref();
            let prev = node.prev.as_ref();

            // if the 'next' node exists, change its 'prev' reference to this node's 'prev'
            // reference. Otherwise, make the 'prev' reference None.
            if let Some(next_n) = next {
                if let Some(inside) = Rc::get_mut(&mut next_n.clone()) {
                    inside.prev = match prev.cloned() {
                        Some(x) => Some(x),
                        None => None,
                    };
                }
            } else {
                return
            }
            // if the 'prev' node exists, change its 'next' reference to this node's 'next'
            // reference. Otherwise, make the 'next' reference None.
            if let Some(prev_n) = prev {
                if let Some(inside) = Rc::get_mut(&mut prev_n.clone()) {
                    inside.next = match next.cloned() {
                        Some(x) => Some(x),
                        None => None,
                    };
                }
            } else {
                return
            }

            // remove all the references from this node.
            if let Some(curr_inside) = Rc::get_mut(&mut node) {
                curr_inside.next = None;
                curr_inside.prev = None;
            } else {
                return
            }


            self.len -= 1;
            drop(node)
        }
    }
}
