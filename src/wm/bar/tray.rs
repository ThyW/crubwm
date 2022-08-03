use cairo::Context;

use crate::{
    config::{IconTraySettings, WmResult},
    wm::geometry::Geometry,
};

#[derive(Clone, Debug)]
pub struct IconTraySegment {
    _icons: Vec<u32>,
    _settings: IconTraySettings,
}

impl IconTraySegment {
    pub fn draw(
        &self,
        _cr: &Context,
        _position: Option<(f32, f32)>,
        _geometry: Geometry,
    ) -> WmResult {
        Ok(())
    }
}

impl From<IconTraySettings> for IconTraySegment {
    fn from(s: IconTraySettings) -> Self {
        Self {
            _icons: Vec::new(),
            _settings: s,
        }
    }
}
