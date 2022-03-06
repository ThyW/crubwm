use std::rc::Rc;

use super::geometry::Geometry;

pub type Client = u32;

pub enum Container {
    Empty,
    Client(Client),
}

struct ContainerNode {
    geometry: Geometry,
    container_type: Container,
}

pub struct ContainerList {
    first: Rc<ContainerNode>,
    last: Rc<ContainerNode>,
}
