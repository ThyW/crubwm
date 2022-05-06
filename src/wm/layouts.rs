use crate::{
    errors::WmResult,
    wm::{
        container::{Container, ContainerType},
        geometry::Geometry,
    },
};

use std::rc::Rc;
use x11rb::{protocol::xproto::ConnectionExt, rust_connection::RustConnection};

pub struct LayoutMask;

impl LayoutMask {
    pub const TILING_EQUAL_HORIZONTAL: u64 = 1 << 0;
    pub const TILING_EQUAL_VERTICAL: u64 = 1 << 1;
    pub const ALL: u64 = LayoutMask::TILING_EQUAL_HORIZONTAL | LayoutMask::TILING_EQUAL_VERTICAL;
}

pub(crate) trait Layout<'a> {
    fn apply<G: Into<Geometry>>(
        &self,
        screen: G,
        cs: std::collections::vec_deque::IterMut<Container>,
        connection: Rc<RustConnection>,
    ) -> WmResult;
}

#[allow(unused)]
#[derive(Clone, Copy)]
pub enum LayoutType {
    TilingEqualHorizontal = LayoutMask::TILING_EQUAL_HORIZONTAL as isize,
    TilingEqualVertical = LayoutMask::TILING_EQUAL_VERTICAL as isize,
}

impl LayoutType {
    pub fn default() -> Self {
        Self::TilingEqualHorizontal
    }
}

impl TryFrom<u64> for LayoutType {
    type Error = crate::errors::Error;

    fn try_from(n: u64) -> WmResult<Self> {
        match n {
            LayoutMask::TILING_EQUAL_HORIZONTAL => Ok(Self::TilingEqualHorizontal),
            LayoutMask::TILING_EQUAL_VERTICAL => Ok(Self::TilingEqualVertical),
            _ => Err("layout error: invalid layout id.".into()),
        }
    }
}

impl TryFrom<&str> for LayoutType {
    type Error = crate::errors::Error;
    fn try_from(str: &str) -> Result<Self, Self::Error> {
        match str.to_lowercase().as_str() {
            "tilingequalhorizontal" => Ok(Self::TilingEqualHorizontal),
            "tilingequalvertical" => Ok(Self::TilingEqualVertical),
            _ => {
                return Err(
                    format!("layout error: {str} is not recognized as a valid layout.").into(),
                )
            }
        }
    }
}

impl<'a> Layout<'a> for LayoutType {
    fn apply<G: Into<Geometry>>(
        &self,
        screen: G,
        cs: std::collections::vec_deque::IterMut<Container>,
        connection: Rc<RustConnection>,
    ) -> WmResult {
        match &self {
            Self::TilingEqualHorizontal => {
                let len = cs.len();
                if len == 0 {
                    return Ok(());
                }
                let screen = screen.into();

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
                            connection.configure_window(c.window_id(), &c.geometry().into())?;
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

                let screen = screen.into();

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
                            connection.configure_window(c.window_id(), &c.geometry().into())?;
                        }
                    }
                }

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn isize_u64() {
        let num: u64 = isize::max_value() as u64;
        let num = num | 1 << 63;
        assert!(num == u64::max_value())
    }
}
