use std::{
    process::Command,
    rc::Rc,
    time::{Instant, UNIX_EPOCH},
    vec,
};

use cairo::Context;

use crate::{
    config::{WidgetSettings, WmResult},
    wm::geometry::TextExtents,
};

#[derive(Clone, Debug)]
pub struct WidgetSegment {
    widgets: Vec<Widget>,
}

#[derive(Clone, Debug)]
pub struct Widget {
    value: String,
    last_update: u64,
    settings: WidgetSettings,
}

impl Widget {
    pub fn update(&mut self) -> WmResult {
        let now = UNIX_EPOCH.elapsed()?.as_secs();

        if now - self.last_update >= self.settings.update_time as u64 || self.last_update == 0 {
            self.value = String::from_utf8(
                Command::new("/bin/sh")
                    .args(["-c", &self.settings.command])
                    .output()?
                    .stdout,
            )?;
            self.last_update = now
        }

        Ok(())
    }

    fn value_with_separator(&self) -> (String, String) {
        (
            format!("{} {}", self.settings.icon, self.value),
            self.settings.separator.clone(),
        )
    }

    fn get_extent_info(&self) -> (String, String) {
        let (value, separator) = self.value_with_separator();
        let text = format!("{} {} {}", separator, value, separator);
        (text, self.settings.font.clone())
    }
}

impl From<Vec<WidgetSettings>> for WidgetSegment {
    fn from(ws: Vec<WidgetSettings>) -> Self {
        let mut ret = vec![];
        for widget_settings in ws {
            ret.push(Widget {
                value: "".to_string(),
                last_update: 0,
                settings: widget_settings,
            })
        }
        Self { widgets: ret }
    }
}

impl WidgetSegment {
    pub fn run_updates(&mut self) -> WmResult {
        for widget in self.widgets.iter_mut() {
            widget.update()?
        }
        Ok(())
    }

    pub fn get_text(&self) -> String {
        let mut buffer = String::new();
        let mut last_sep = String::new();

        for widget in self.widgets.iter() {
            let value = &widget.value_with_separator();
            buffer.push_str(&value.1);
            buffer.push_str(&value.0);
            last_sep = value.1.clone();
        }
        buffer.push_str(&last_sep);

        buffer
    }

    pub fn get_text_extents(&self, cr: &Context, font_size: f64) -> WmResult<TextExtents> {
        let mut extents = TextExtents::default();

        for widget in self.widgets.iter() {
            let (txt, font) = widget.get_extent_info();

            cr.set_font_size(font_size);
            cr.select_font_face(&font, cairo::FontSlant::Normal, cairo::FontWeight::Normal);
            let ext: TextExtents = cr.text_extents(&txt)?.into();
            extents += ext;
        }

        Ok(extents)
    }
}
