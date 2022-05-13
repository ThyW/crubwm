use crate::{
    errors::WmResult,
    wm::{
        container::{Container, ContainerType},
        geometry::Geometry,
    },
};

use std::rc::Rc;
use x11rb::{
    protocol::xproto::{ConfigureWindowAux, ConnectionExt},
    rust_connection::RustConnection,
};

pub struct LayoutMask;

impl LayoutMask {
    pub const TILING_EQUAL_HORIZONTAL: u64 = 1 << 0;
    pub const TILING_EQUAL_VERTICAL: u64 = 1 << 1;
    pub const TILING_MASTER_STACK: u64 = 1 << 2;
    pub const ALL: u64 = LayoutMask::TILING_EQUAL_HORIZONTAL
        | LayoutMask::TILING_EQUAL_VERTICAL
        | LayoutMask::TILING_MASTER_STACK;

    pub fn from_slice(slice: &Vec<String>) -> WmResult<u64> {
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

pub(crate) trait Layout<'a> {
    fn apply<G: Into<Geometry>>(
        &self,
        screen: G,
        cs: (usize, std::collections::vec_deque::IterMut<Container>),
        connection: Rc<RustConnection>,
    ) -> WmResult;
}

#[allow(unused)]
#[derive(Clone, Copy)]
pub enum LayoutType {
    TilingEqualHorizontal = LayoutMask::TILING_EQUAL_HORIZONTAL as isize,
    TilingEqualVertical = LayoutMask::TILING_EQUAL_VERTICAL as isize,
    TilingMasterStack = LayoutMask::TILING_MASTER_STACK as isize,
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
    fn apply<G: Into<Geometry>>(
        &self,
        screen: G,
        cs: (usize, std::collections::vec_deque::IterMut<Container>),
        connection: Rc<RustConnection>,
    ) -> WmResult {
        match &self {
            Self::TilingEqualHorizontal => {
                println!("horizontal");
                let (len, iter) = cs;
                if len == 0 {
                    return Ok(());
                }
                let screen = screen.into();

                let width = screen.width / len as u16;
                let mut ii = -1;

                for each in iter.into_iter() {
                    match each.data_mut() {
                        ContainerType::Empty(g) => {
                            ii += 1;
                            g.y = 0;
                            g.x = width as i16 * ii as i16;
                            g.width = width;
                            g.height = screen.height;
                        }
                        ContainerType::InLayout(c) => {
                            ii += 1;
                            c.geometry.y = 0;
                            c.geometry.x = width as i16 * ii as i16;
                            c.geometry.width = width;
                            c.geometry.height = screen.height;
                            let aux: ConfigureWindowAux = c.geometry().into();
                            connection.configure_window(c.window_id(), &aux)?;
                        }
                        ContainerType::Floating(_) => (),
                    };
                }

                Ok(())
            }
            Self::TilingEqualVertical => {
                println!("vertical");
                let (len, iter) = cs;
                if len == 0 {
                    return Ok(());
                }

                let screen = screen.into();

                let height = screen.height / len as u16;
                let mut ii = -1;

                for each in iter.into_iter() {
                    match each.data_mut() {
                        ContainerType::Floating(_) => (),
                        ContainerType::Empty(g) => {
                            ii += 1;
                            g.x = 0;
                            g.y = height as i16 * ii as i16;
                            g.width = screen.width;
                            g.height = height;
                        }
                        ContainerType::InLayout(c) => {
                            ii += 1;
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
            Self::TilingMasterStack => {
                println!("master stack");
                let (len, iter) = cs;
                if len == 0 {
                    return Ok(());
                }

                let screen: Geometry = screen.into();
                if len == 1 {
                    for one in iter.into_iter() {
                        match one.data_mut() {
                            ContainerType::Empty(g) => {
                                g.x = 0;
                                g.y = 0;
                                g.width = screen.width;
                                g.height = screen.height;
                            }
                            ContainerType::InLayout(c) => {
                                c.geometry.x = 0;
                                c.geometry.y = 0;
                                c.geometry.width = screen.width;
                                c.geometry.height = screen.height;
                                connection.configure_window(c.window_id(), &c.geometry().into())?;
                            }
                            _ => {}
                        };
                    }
                    Ok(())
                } else {
                    let height = screen.height / (len - 1) as u16;
                    let width = screen.width / 2;

                    let mut ii = -2;
                    for each in iter.into_iter() {
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
                                    c.geometry.x = 0;
                                    c.geometry.y = 0;
                                    c.geometry.width = screen.width / 2;
                                    c.geometry.height = screen.height;
                                    connection
                                        .configure_window(c.window_id(), &c.geometry().into())?;
                                } else {
                                    c.geometry.x = width as i16 - 1;
                                    c.geometry.y = height as i16 * ii;
                                    c.geometry.width = screen.width / 2;
                                    c.geometry.height = height;
                                    connection
                                        .configure_window(c.window_id(), &c.geometry().into())?;
                                }
                            }
                            _ => {}
                        }
                    }

                    Ok(())
                }
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
