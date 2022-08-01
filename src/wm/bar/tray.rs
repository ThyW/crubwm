use cairo::Context;

use crate::config::{IconTraySettings, WmResult};

#[derive(Clone, Debug)]
pub struct IconTraySegment {
    icons: Vec<u32>,
    settings: IconTraySettings,
}

impl IconTraySegment {
    pub fn draw(&self, cr: &Context, position: Option<(f32, f32)>) -> WmResult {
        Ok(())
    }
}

impl From<IconTraySettings> for IconTraySegment {
    fn from(s: IconTraySettings) -> Self {
        Self {
            icons: Vec::new(),
            settings: s,
        }
    }
}
