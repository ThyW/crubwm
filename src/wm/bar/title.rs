use cairo::Context;

use crate::{
    config::{WindowTitleSettings, WmResult},
    wm::geometry::TextExtents,
};

#[derive(Clone, Debug)]
pub struct TitlebarSegment {
    title: String,
    settings: WindowTitleSettings,
}

impl TitlebarSegment {
    pub fn set_title(&mut self, title: String) {
        self.title = title
    }

    pub fn get_text(&self) -> String {
        self.title.clone()
    }

    pub fn get_text_extent(&self, cr: &Context, font_size: f64) -> WmResult<TextExtents> {
        cr.select_font_face(
            &self.settings.font,
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal,
        );
        cr.set_font_size(font_size);
        Ok(cr.text_extents(&self.get_text())?.into())
    }
}

impl From<WindowTitleSettings> for TitlebarSegment {
    fn from(s: WindowTitleSettings) -> Self {
        Self {
            title: "".to_string(),
            settings: s,
        }
    }
}
