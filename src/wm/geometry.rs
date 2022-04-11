use x11rb::protocol::xproto::ConfigureWindowAux;
use x11rb::protocol::xproto::GetGeometryReply;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Geometry {
    pub root: u32,
    pub depth: u8,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub border_width: u16,
    pub height: u16,
}

impl From<GetGeometryReply> for Geometry {
    fn from(g: GetGeometryReply) -> Self {
        Self {
            root: g.root,
            depth: g.depth,
            x: g.x,
            y: g.y,
            width: g.width,
            border_width: g.border_width,
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
            .height(Some(g.width as u32))
            .border_width(Some(g.border_width as u32))
    }
}
