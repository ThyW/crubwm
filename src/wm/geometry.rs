use crate::config::Config;
use x11rb::protocol::xproto::{ConfigureWindowAux, GetGeometryReply};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Geometry {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClientAttributes {
    pub gap_top: u32,
    pub gap_bottom: u32,
    pub gap_left: u32,
    pub gap_right: u32,

    pub border_size: u32,
    pub border_color: u32,
}

impl std::fmt::Display for Geometry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x: {} ", self.x).and_then(|_| {
            write!(f, "y: {} ", self.y).and_then(|_| {
                write!(f, "w: {} ", self.width).and_then(|_| write!(f, "h: {} ", self.height))
            })
        })
    }
}

impl From<GetGeometryReply> for Geometry {
    fn from(g: GetGeometryReply) -> Self {
        Self {
            x: g.x,
            y: g.y,
            width: g.width,
            height: g.height,
        }
    }
}

impl From<Geometry> for ConfigureWindowAux {
    fn from(g: Geometry) -> Self {
        ConfigureWindowAux::new()
            .x(Some(g.x as i32))
            .y(Some(g.y as i32))
            .width(Some(g.width as u32))
            .height(Some(g.height as u32))
            .border_width(Some(0u32))
    }
}

impl From<Config> for ClientAttributes {
    fn from(c: Config) -> Self {
        let gaps = c.options.get_gaps();
        let border = c.options.get_borders();
        let border_color = c.options.convert_border_color();
        Self {
            gap_top: gaps.0,
            gap_bottom: gaps.1,
            gap_left: gaps.2,
            gap_right: gaps.3,
            border_size: border,
            border_color,
        }
    }
}
