use cairo::Context;

use crate::{
    config::WindowTitleSettings,
    errors::WmResult,
    utils,
    wm::geometry::{Geometry, TextExtents},
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

    pub fn get_text_extent(&self, cr: &Context, font_size: Option<f64>) -> WmResult<TextExtents> {
        utils::cairo_font_from_str(cr, &self.settings.font)?;
        if let Some(size) = font_size {
            cr.set_font_size(size);
        }
        Ok(cr.text_extents(&self.get_text())?.into())
    }

    pub fn draw(&self, cr: &Context, position: Option<(f32, f32)>, geometry: Geometry) -> WmResult {
        if let Some((x, y)) = position {
            cr.move_to(x.into(), y.into());
        }

        utils::cairo_font_from_str(cr, &self.settings.font)?;

        let extents = self.get_text_extent(cr, None)?;

        let (x, y) = cr.current_point()?;
        let (r, g, b) = utils::translate_color(self.settings.background_color.clone())?;
        cr.set_source_rgb(r, g, b);
        cr.rectangle(x, 0., extents.width, geometry.height as _);
        cr.fill()?;

        let text = self.get_text();
        cr.move_to(x, y);
        let (r, g, b) = utils::translate_color(self.settings.foreground_color.clone())?;
        cr.set_source_rgb(r, g, b);
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
