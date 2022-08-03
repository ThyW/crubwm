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
