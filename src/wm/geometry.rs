use std::ops::{Add, AddAssign};

use crate::config::Config;
use x11rb::protocol::xproto::{ConfigureWindowAux, GetGeometryReply};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Geometry {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

impl Geometry {
    pub fn minus_bar(self, other: Self) -> Self {
        // +--------------------------------------+
        // |xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx|
        // |xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx|
        // |xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx|
        // |--------------------------------------|
        // |*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x|
        // |*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x|
        // |*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x|
        // |*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x|
        // |*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x|
        // |*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x|
        // |*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x|
        // |*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x|
        // |--------------------------------------|
        // |**************************************|
        // |--------------------------------------+
        let mut height = self.height;
        let width = self.width;

        let mut y_0 = self.y;

        if y_0 == other.y {
            height -= other.height;
            y_0 += other.height as i16;
        } else {
            height -= other.height
        }

        Self {
            x: self.x,
            y: y_0,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClientAttributes {
    pub gap_top: u32,
    pub gap_bottom: u32,
    pub gap_left: u32,
    pub gap_right: u32,

    pub border_size: u32,
    pub border_color: u32,
}

impl std::fmt::Display for Geometry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x: {} ", self.x).and_then(|_| {
            write!(f, "y: {} ", self.y).and_then(|_| {
                write!(f, "w: {} ", self.width).and_then(|_| write!(f, "h: {} ", self.height))
            })
        })
    }
}

impl From<GetGeometryReply> for Geometry {
    fn from(g: GetGeometryReply) -> Self {
        Self {
            x: g.x,
            y: g.y,
            width: g.width,
            height: g.height,
        }
    }
}

impl From<Geometry> for ConfigureWindowAux {
    fn from(g: Geometry) -> Self {
        ConfigureWindowAux::new()
            .x(Some(g.x as i32))
            .y(Some(g.y as i32))
            .width(Some(g.width as u32))
            .height(Some(g.height as u32))
            .border_width(Some(0u32))
    }
}

impl From<Config> for ClientAttributes {
    fn from(c: Config) -> Self {
        let gaps = c.options.get_gaps();
        let border = c.options.get_borders();
        let border_color = c.options.convert_border_color();
        Self {
            gap_top: gaps.0,
            gap_bottom: gaps.1,
            gap_left: gaps.2,
            gap_right: gaps.3,
            border_size: border,
            border_color,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub struct TextExtents {
    pub width: f64,
    pub height: f64,
    pub advance: f64,
    pub bearing: f64,
}

impl From<cairo::TextExtents> for TextExtents {
    fn from(o: cairo::TextExtents) -> Self {
        Self {
            width: o.width,
            height: o.height,
            advance: o.x_advance,
            bearing: o.x_bearing,
        }
    }
}

impl TextExtents {
    pub fn _new<I: Into<f64>>(width: I, height: I, advance: I, bearing: I) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
            advance: advance.into(),
            bearing: bearing.into(),
        }
    }
}

impl Add for TextExtents {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let width = self.width + rhs.width;
        let mut height = self.height;
        if rhs.height > self.height {
            height = rhs.height;
        }

        Self {
            width,
            height,
            advance: self.advance + rhs.advance,
            bearing: self.bearing + rhs.bearing,
        }
    }
}

impl AddAssign for TextExtents {
    fn add_assign(&mut self, rhs: Self) {
        let other = *self + rhs;
        self.width = other.width;
        self.height = other.height;
        self.advance = other.advance;
        self.bearing = other.bearing;
    }
}

impl std::ops::Sub for Geometry {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: rhs.x,
            y: rhs.height as _,
            width: self.width - rhs.x as u16,
            height: self.height - rhs.height,
        }
    }
}
