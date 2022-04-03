use std::rc::Rc;
use std::cell::RefCell;

use crate::errors::WmResult;
use crate::wm::container::{Container, ContainerListNode};
use crate::wm::geometry::Geometry;

pub(crate) trait Layout<'a> {
    fn apply(&self, screen: Geometry, cs: Vec<Rc<RefCell<ContainerListNode>>>) -> WmResult;
}

pub enum LayoutType {
    TilingEqualHorizontal,
    TilingEqualVertical,
}

impl LayoutType {
    pub fn default() -> Self {
        Self::TilingEqualHorizontal
    }
}

impl<'a> Layout<'a> for LayoutType {
    fn apply(&self, screen: Geometry, cs: Vec<Rc<RefCell<ContainerListNode>>>) -> WmResult {
        if cs.is_empty() {
            return Ok(())
        }

        match &self {
            Self::TilingEqualHorizontal => {
                let len = cs.len();
                let width = screen.width / len as u16;

                for (ii, rc) in cs.into_iter().enumerate() {
                    let mut node = rc.try_borrow_mut()?;
                    let each = node.data_mut();
                    match each {
                        Container::Empty(g) => {
                            g.y = 0;
                            g.x = width as i16 * ii as i16;
                            g.width = width;
                            g.height = screen.height;
                        }
                        Container::InLayout(c) => {
                            c.geometry.y = 0;
                            c.geometry.x = width as i16 * ii as i16;
                            c.geometry.width = width;
                            c.geometry.height = screen.height;
                        }
                        Container::Floating(_) => (),
                    };
                }
                Ok(())
            }
            Self::TilingEqualVertical => {
                let len = cs.len();
                let height = screen.height / len as u16;

                for (ii, rc) in cs.into_iter().enumerate() {
                    let mut node = rc.try_borrow_mut()?;
                    let each = node.data_mut();
                    match each {
                        Container::Floating(_) => (),
                        Container::Empty(g) => {
                            g.x = 0;
                            g.y = height as i16 * ii as i16;
                            g.width = screen.width;
                            g.height = height;
                        }
                        Container::InLayout(c) => {
                            c.geometry.x = 0;
                            c.geometry.y = height as i16 * ii as i16;
                            c.geometry.width = screen.width;
                            c.geometry.height = height;
                        }
                    }
                }

                Ok(())
            }
        }
    }
}
