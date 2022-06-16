use crate::{
    errors::WmResult,
    wm::{
        container::{Container, ContainerType},
        geometry::Geometry,
    },
};

use std::rc::Rc;
use x11rb::protocol::xproto::ConnectionExt;

pub struct LayoutMask;

impl LayoutMask {
    pub const TILING_EQUAL_HORIZONTAL: u64 = 1 << 0;
    pub const TILING_EQUAL_VERTICAL: u64 = 1 << 1;
    pub const TILING_MASTER_STACK: u64 = 1 << 2;
    pub const STACKING_HORIZONTAL: u64 = 1 << 3;
    pub const ALL: u64 = LayoutMask::TILING_EQUAL_HORIZONTAL
        | LayoutMask::TILING_EQUAL_VERTICAL
        | LayoutMask::TILING_MASTER_STACK
        | LayoutMask::STACKING_HORIZONTAL;

    pub fn from_slice(slice: &[String]) -> WmResult<u64> {
        let mut mask = 0u64;
        for each in slice.iter() {
            let eeach = each.clone().to_lowercase();
            if eeach == "none" {
                return Ok(0);
            } else if eeach == "all" {
                return Ok(Self::ALL);
            } else {
                let layout = LayoutType::try_from(eeach.as_str())?;
                mask |= layout as u64
            }
        }
        Ok(mask)
    }
}

pub trait Layout<'a> {
    fn apply<G: Into<Geometry>, C: x11rb::connection::Connection, I: Into<u32>>(
        &self,
        screen: G,
        cs: (usize, std::collections::vec_deque::IterMut<Container>),
        connection: Rc<C>,
        default_colormap: I,
        focused_client: Option<u32>,
    ) -> WmResult;
}

#[allow(clippy::enum_variant_names)]
#[allow(unused)]
#[repr(u64)]
#[derive(Clone, Copy)]
pub enum LayoutType {
    TilingEqualHorizontal = LayoutMask::TILING_EQUAL_HORIZONTAL,
    TilingEqualVertical = LayoutMask::TILING_EQUAL_VERTICAL,
    TilingMasterStack = LayoutMask::TILING_MASTER_STACK,
    StackingHorizontal = LayoutMask::STACKING_HORIZONTAL,
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
            LayoutMask::TILING_MASTER_STACK => Ok(Self::TilingMasterStack),
            LayoutMask::STACKING_HORIZONTAL => Ok(Self::StackingHorizontal),
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
            "tilingmasterstack" => Ok(Self::TilingMasterStack),
            _ => {
                return Err(
                    format!("layout error: \"{str}\" is not recognized as a valid layout.").into(),
                )
            }
        }
    }
}

impl<'a> Layout<'a> for LayoutType {
    fn apply<G: Into<Geometry>, C: x11rb::connection::Connection, I: Into<u32>>(
        &self,
        screen: G,
        cs: (usize, std::collections::vec_deque::IterMut<Container>),
        connection: Rc<C>,
        default_colormap: I,
        focused_clinet: Option<u32>,
    ) -> WmResult {
        let default_colormap = default_colormap.into();
        match &self {
            Self::TilingEqualHorizontal => {
                let (len, iter) = cs;
                if len == 0 {
                    return Ok(());
                }
                let screen = screen.into();

                let width = screen.width / len as u16;
                let mut ii = -1;

                for each in iter {
                    match each.data_mut() {
                        ContainerType::Empty(g) => {
                            ii += 1;
                            g.y = screen.y;
                            g.x = screen.x + (width as i16 * ii as i16);
                            g.width = width;
                            g.height = screen.height;
                        }
                        ContainerType::InLayout(c) => {
                            ii += 1;
                            c.geometry.x = screen.x + (width as i16 * ii as i16);
                            c.geometry.y = screen.y;
                            c.geometry.width = width;
                            c.geometry.height = screen.height;
                            c.draw_borders(connection.clone(), default_colormap)?;
                        }
                        ContainerType::Floating(_) => (),
                    };
                }

                Ok(())
            }
            Self::TilingEqualVertical => {
                let (len, iter) = cs;
                if len == 0 {
                    return Ok(());
                }

                let screen = screen.into();

                let height = screen.height / len as u16;
                let mut ii = -1;

                for each in iter {
                    match each.data_mut() {
                        ContainerType::Floating(_) => (),
                        ContainerType::Empty(g) => {
                            ii += 1;
                            g.x = screen.x;
                            g.y = screen.y + height as i16 * ii as i16;
                            g.width = screen.width;
                            g.height = height;
                        }
                        ContainerType::InLayout(c) => {
                            ii += 1;
                            c.geometry.x = screen.x;
                            c.geometry.y = screen.y + (height as i16 * ii as i16);
                            c.geometry.width = screen.width;
                            c.geometry.height = height;
                            c.draw_borders(connection.clone(), default_colormap)?;
                        }
                    }
                }

                Ok(())
            }
            Self::TilingMasterStack => {
                let (len, iter) = cs;
                if len == 0 {
                    return Ok(());
                }

                let screen: Geometry = screen.into();
                if len == 1 {
                    for one in iter {
                        match one.data_mut() {
                            ContainerType::Empty(g) => {
                                g.x = screen.x;
                                g.y = screen.y;
                                g.width = screen.width;
                                g.height = screen.height;
                            }
                            ContainerType::InLayout(c) => {
                                c.geometry.x = screen.x;
                                c.geometry.y = screen.y;
                                c.geometry.width = screen.width;
                                c.geometry.height = screen.height;
                                c.draw_borders(connection.clone(), default_colormap)?;
                            }
                            _ => {}
                        };
                    }
                    Ok(())
                } else {
                    let height = screen.height / (len - 1) as u16;
                    let width = screen.width / 2;

                    let mut ii = -2;
                    for each in iter {
                        match each.data_mut() {
                            ContainerType::Empty(g) => {
                                ii += 1;
                                if ii == -1 {
                                    g.x = 0;
                                    g.y = 0;
                                    g.width = width;
                                    g.height = screen.height;
                                } else {
                                    g.x = width as i16;
                                    g.y = height as i16 * ii;
                                    g.width = width;
                                    g.height = height;
                                }
                            }
                            ContainerType::InLayout(c) => {
                                ii += 1;
                                if ii == -1 {
                                    c.geometry.x = screen.x;
                                    c.geometry.y = screen.y;
                                    c.geometry.width = screen.width / 2;
                                    c.geometry.height = screen.height;
                                    c.draw_borders(connection.clone(), default_colormap)?;
                                } else {
                                    c.geometry.x = screen.x + width as i16 - 1;
                                    c.geometry.y = screen.y + height as i16 * ii;
                                    c.geometry.width = screen.width / 2;
                                    c.geometry.height = height;
                                    c.draw_borders(connection.clone(), default_colormap)?;
                                }
                            }
                            _ => {}
                        }
                    }

                    Ok(())
                }
            }
            Self::StackingHorizontal => {
                let screen = screen.into();
                if cs.0 == 0 {
                    return Ok(());
                }

                for container in cs.1.into_iter() {
                    match container.data_mut() {
                        ContainerType::InLayout(c) => {
                            if let Some(focused_client) = focused_clinet {
                                if focused_client == c.window_id() {
                                    c.geometry.x = screen.x;
                                    c.geometry.y = screen.y;
                                    c.geometry.width = screen.width;
                                    c.geometry.height = screen.height;

                                    c.draw_borders(connection.clone(), default_colormap)?;
                                    connection.map_subwindows(c.window_id())?;
                                    connection.map_window(c.window_id())?;
                                } else {
                                    c.geometry.x = screen.x;
                                    c.geometry.y = screen.y;
                                    c.geometry.width = screen.width;
                                    c.geometry.height = screen.height;

                                    connection.unmap_subwindows(c.window_id())?;
                                    connection.unmap_window(c.window_id())?;
                                }
                            }
                        }
                        _ => (),
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
