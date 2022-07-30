use x11rb::{connection::Connection, protocol::xproto::Visualtype};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct xcb_visualtype_t {
    pub visual_id: u32,
    pub class: u8,
    pub bits_per_rgb_value: u8,
    pub colormap_entries: u16,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
    pub pad0: [u8; 4],
}

impl From<Visualtype> for xcb_visualtype_t {
    fn from(a: Visualtype) -> Self {
        Self {
            visual_id: a.visual_id,
            class: u8::from(a.class),
            bits_per_rgb_value: a.bits_per_rgb_value,
            colormap_entries: a.colormap_entries,
            red_mask: a.red_mask,
            green_mask: a.green_mask,
            blue_mask: a.blue_mask,
            pad0: [0; 4],
        }
    }
}

pub fn find_xcb_visualtype(conn: &impl Connection, visual_id: u32) -> Option<xcb_visualtype_t> {
    for root in &conn.setup().roots {
        for depth in &root.allowed_depths {
            for visual in &depth.visuals {
                if visual.visual_id == visual_id {
                    return Some((*visual).into());
                }
            }
        }
    }
    None
}
