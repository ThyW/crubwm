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

    pub fn draw(&self, cr: &Context, position: Option<(f32, f32)>) -> WmResult {
        if let Some((x, y)) = position {
            cr.move_to(x.into(), y.into());
        }

        cr.select_font_face(
            &self.settings.font,
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal,
        );
        let text = self.get_text();
        cr.set_source_rgb(1., 1., 1.);
        cr.show_text(&text)?;

        Ok(())
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
