use std::{process::Command, time::UNIX_EPOCH, vec};

use cairo::Context;

use crate::{
    config::{WidgetSettings, WmResult},
    errors::Error,
    utils,
    wm::geometry::{Geometry, TextExtents},
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

#[derive(Debug, Clone)]
enum FormatToken {
    Literal(char),
    Icon(String),
    Value(String),
    Separator(String),
}

impl FormatToken {
    fn text(&self) -> String {
        match self {
            FormatToken::Icon(s) => s.clone(),
            FormatToken::Value(s) => s.clone(),
            FormatToken::Separator(s) => s.clone(),
            FormatToken::Literal(s) => s.to_string(),
        }
    }
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
            )?
            .trim()
            .to_string();
            self.last_update = now
        }

        Ok(())
    }

    fn _value_with_separator(&self) -> (String, String) {
        (
            format!("{} {}", self.settings.icon, self.value),
            self.settings.separator.clone(),
        )
    }

    fn _value(&self) -> WmResult<Vec<FormatToken>> {
        let mut output: Vec<FormatToken> = Vec::new();
        let fmt = self.settings.format.clone();

        let mut in_brace = false;
        let mut brace_value = String::new();

        for char in fmt.chars() {
            if !in_brace {
                if char == '{' {
                    in_brace = true;
                    continue;
                };
                output.push(FormatToken::Literal(char))
            } else if char == '}' {
                in_brace = false;
                match &brace_value[..] {
                    "icon" => output.push(FormatToken::Icon(self.settings.icon.clone())),
                    "value" => output.push(FormatToken::Value(self.value.clone())),
                    "separator" => {
                        output.push(FormatToken::Separator(self.settings.separator.clone()))
                    }
                    _ => (),
                };
                brace_value.clear()
            } else {
                brace_value.push(char)
            }
        }

        if in_brace {
            return Err(Error::Generic(format!("{fmt} is missing a closing brace.")));
        }

        Ok(output)
    }

    fn get_extent_info(&self, cr: &Context) -> WmResult<TextExtents> {
        /* let (value, separator) = self.value_with_separator();
        let text = format!("{}-{}-{}", separator, value, separator);
        (text, self.settings.font.clone()) */

        let mut extents = TextExtents::default();
        let tokens = self._value()?;

        for token in tokens.iter() {
            let text = token.text();
            let text = &if &text[..] == " " {
                "-".to_string()
            } else {
                text
            };
            let ext = cr.text_extents(text)?;
            extents += ext.into();
        }

        Ok(extents)
    }

    fn draw(
        &self,
        cr: &Context,
        position: Option<(f64, f64)>,
        geometry: Geometry,
    ) -> WmResult<f64> {
        cr.select_font_face(
            &self.settings.font,
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal,
        );

        if let Some((x, y)) = position {
            cr.move_to(x, y)
        }

        let tokens = self._value()?;

        let extents: TextExtents = self.get_extent_info(cr)?;
        let (x, y) = cr.current_point()?;

        let (r, g, b) = utils::translate_color(self.settings.background_color.clone())?;
        cr.set_source_rgb(r, g, b);
        cr.rectangle(x, 0., extents.advance, geometry.height as _);
        cr.fill()?;

        cr.move_to(x, y);

        for token in tokens.iter() {
            match token.clone() {
                FormatToken::Literal(_) => {
                    let (r, g, b) = utils::translate_color(self.settings.separator_color.clone())?;
                    cr.set_source_rgb(r, g, b);
                    cr.show_text(token.text().as_str())?;
                }
                FormatToken::Icon(_) => {
                    let (r, g, b) = utils::translate_color(self.settings.icon_color.clone())?;
                    cr.set_source_rgb(r, g, b);
                    cr.show_text(token.text().as_str())?;
                }
                FormatToken::Value(_) => {
                    let (r, g, b) = utils::translate_color(self.settings.value_color.clone())?;
                    cr.set_source_rgb(r, g, b);
                    cr.show_text(token.text().as_str())?;
                }
                FormatToken::Separator(_) => {
                    let (r, g, b) = utils::translate_color(self.settings.separator_color.clone())?;
                    cr.set_source_rgb(r, g, b);
                    cr.show_text(token.text().as_str())?;
                }
            }
        }

        Ok(extents.width)
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

    pub fn _get_text(&self) -> String {
        let mut buffer = String::new();
        let mut last_sep = String::new();

        for widget in self.widgets.iter() {
            let value = &widget._value_with_separator();
            buffer.push_str(&value.1);
            buffer.push_str(&value.0);
            last_sep = value.1.clone();
        }
        buffer.push_str(&last_sep);

        buffer
    }

    pub fn get_text_extents(&self, cr: &Context, font_size: f64) -> WmResult<TextExtents> {
        let mut extents = TextExtents::default();

        cr.set_font_size(font_size);
        for widget in self.widgets.iter() {
            let ext = widget.get_extent_info(cr)?;
            extents += ext;
        }

        Ok(extents)
    }

    pub fn draw(&self, cr: &Context, position: Option<(f32, f32)>, geometry: Geometry) -> WmResult {
        // should draw a backgroud too
        if let Some((x, y)) = position {
            cr.move_to(x.into(), y.into())
        }

        for widget in self.widgets.iter() {
            widget.draw(cr, None, geometry)?;
        }
        Ok(())
    }
}
