use crate::errors::WmResult;
use crate::wm::container::{Container, ContainerType};
use crate::wm::geometry::Geometry;

pub(crate) trait Layout<'a> {
    fn apply(
        &self,
        screen: Geometry,
        cs: std::collections::vec_deque::IterMut<Container>,
    ) -> WmResult;
}

#[allow(unused)]
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
    fn apply(
        &self,
        screen: Geometry,
        cs: std::collections::vec_deque::IterMut<Container>,
    ) -> WmResult {
        match &self {
            Self::TilingEqualHorizontal => {
                let len = cs.len();
                if len == 0 {
                    return Ok(());
                }

                let width = screen.width / len as u16;

                for (ii, each) in cs.into_iter().enumerate() {
                    match each.data_mut() {
                        ContainerType::Empty(g) => {
                            g.y = 0;
                            g.x = width as i16 * ii as i16;
                            g.width = width;
                            g.height = screen.height;
                        }
                        ContainerType::InLayout(c) => {
                            c.geometry.y = 0;
                            c.geometry.x = width as i16 * ii as i16;
                            c.geometry.width = width;
                            c.geometry.height = screen.height;
                            println!("geom: {}", c.geometry);
                        }
                        ContainerType::Floating(_) => (),
                    };
                }
                Ok(())
            }
            Self::TilingEqualVertical => {
                let len = cs.len();
                if len == 0 {
                    return Ok(());
                }

                let height = screen.height / len as u16;

                for (ii, each) in cs.into_iter().enumerate() {
                    match each.data_mut() {
                        ContainerType::Floating(_) => (),
                        ContainerType::Empty(g) => {
                            g.x = 0;
                            g.y = height as i16 * ii as i16;
                            g.width = screen.width;
                            g.height = height;
                        }
                        ContainerType::InLayout(c) => {
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
