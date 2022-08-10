use cairo::{Context, FontSlant, FontWeight};

use crate::errors::*;

pub fn translate_color(input: String) -> WmResult<(f64, f64, f64)> {
    let input = input.strip_prefix('#').ok_or_else(|| {
        Error::Generic(format!(
            "workspace settings error: {} is an invalid color.",
            input
        ))
    })?;

    let mut vec: Vec<f64> = vec![];

    for chunks in input.as_bytes().chunks(2) {
        let string = String::from_utf8(chunks.to_vec())?;
        let num = u8::from_str_radix(&string, 16)?;
        let out = num as f64 / 255.;
        vec.push(out)
    }

    let ret = (vec[0], vec[1], vec[2]);

    Ok(ret)
}

pub fn cairo_font_from_str(cr: &Context, font: impl AsRef<str>) -> WmResult {
    let mut weight = FontWeight::Normal;
    let mut slant = FontSlant::Normal;
    let mut new_font = "";
    for part in font.as_ref().split(':') {
        if part.contains('=') {
            let parts: Vec<&str> = part.split('=').collect();
            if !parts.len() == 2 {
                return Err(format!("Invalid font format: {}", font.as_ref()).into());
            } else {
                match parts[0] {
                    "slant" => {
                        let new_slant = match &parts[1].to_lowercase()[..] {
                            "normal" => Some(FontSlant::Normal),
                            "italic" => Some(FontSlant::Italic),
                            "oblique" => Some(FontSlant::Oblique),
                            _ => None,
                        };
                        if let Some(s) = new_slant {
                            slant = s;
                        };
                    }
                    "weight" => {
                        let new_weight = match &parts[1].to_lowercase()[..] {
                            "normal" => Some(FontWeight::Normal),
                            "bold" => Some(FontWeight::Bold),
                            _ => None,
                        };
                        if let Some(s) = new_weight {
                            weight = s;
                        };
                    }
                    _ => return Err(format!("Invalid font property: {}", parts[0]).into()),
                }
            }
        } else {
            new_font = part;
        }
    }

    cr.select_font_face(new_font, slant, weight);

    Ok(())
}
