use crate::errors::WmResult;

#[derive(Debug)]
#[allow(unused)]
#[derive(Clone)]
pub struct Options {
    /// Should a window border be shown on the given side of the window?
    ///
    /// Default: disabled for all
    pub border_up: bool,
    pub border_down: bool,
    pub border_left: bool,
    pub border_right: bool,

    /// Size, in pixels of window borders.
    ///
    /// If the border for the given side is disabled, the value will be ignored.
    /// If the value is 0, the border won't be shown.
    ///
    /// Default: 1 for all
    pub border_up_size: u32,
    pub border_down_size: u32,
    pub border_left_size: u32,
    pub border_right_size: u32,

    /// A hexadecimal RGB representation of the window border color.
    ///
    /// Default: #000000 (full black)
    pub border_color: String,

    /// True by default, render a bar on top of the window to show its name.
    ///
    /// Default: true
    pub show_window_name: bool,
    /// Where in the name bar should a window's name be shown.
    ///
    /// Can be:
    ///     - left: left most part of window name tag
    ///     - middle: in the middle of the winodw name tage
    ///     - right: right most part of the window name tag
    ///
    /// Default: left
    pub window_name_position: String,
    /// The display name to use when connecting to a X11 server.
    ///
    /// Default is an empty string, which tells the WM to use the value from the DISPLAY environmental
    /// variable.
    pub display_name: String,

    /// Should a gap be produced on the given side of the window?
    ///
    /// Default: disable for all
    pub gap_top: bool,
    pub gap_bottom: bool,
    pub gap_left: bool,
    pub gap_right: bool,

    /// Size, in pixels, of the gap between windows on each side.
    ///
    /// If the gap on the given side is disabled, the value will be ignored. Value of 0 implies
    /// that the border should not be shown.
    ///
    /// Default: 0 for all
    pub gap_top_size: u32,
    pub gap_bottom_size: u32,
    pub gap_left_size: u32,
    pub gap_right_size: u32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            border_up: false,
            border_down: false,
            border_left: false,
            border_right: false,

            border_up_size: 1,
            border_down_size: 1,
            border_left_size: 1,
            border_right_size: 1,

            border_color: "#000000".to_string(),

            show_window_name: true,
            window_name_position: "left".to_string(),
            display_name: "".to_string(),

            gap_top: false,
            gap_bottom: false,
            gap_left: false,
            gap_right: false,

            gap_top_size: 0,
            gap_bottom_size: 0,
            gap_left_size: 0,
            gap_right_size: 0,
        }
    }
}

impl Options {
    pub fn add(&mut self, name: String, value: String) -> WmResult {
        match name.as_ref() {
            "border_up" => {
                let val = value.to_lowercase().parse::<bool>()?;
                self.border_up = val;
            }
            "border_down" => {
                let val = value.to_lowercase().parse::<bool>()?;
                self.border_down = val;
            }
            "border_left" => {
                let val = value.to_lowercase().parse::<bool>()?;
                self.border_left = val;
            }
            "border_right" => {
                let val = value.to_lowercase().parse::<bool>()?;
                self.border_right = val;
            }
            "border_up_size" => {
                let val = value.to_lowercase().parse::<u32>()?;
                self.border_up_size = val;
            }
            "border_down_size" => {
                let val = value.to_lowercase().parse::<u32>()?;
                self.border_down_size = val;
            }
            "border_left_size" => {
                let val = value.to_lowercase().parse::<u32>()?;
                self.border_left_size = val;
            }
            "border_right_size" => {
                let val = value.to_lowercase().parse::<u32>()?;
                self.border_right_size = val;
            }
            "border_color" => {
                if value.starts_with('#') && value.len() == 7 {
                    self.border_color = value;
                }
            }
            "show_window_name" => {
                let val = value.to_lowercase().parse::<bool>()?;
                self.show_window_name = val;
            }
            "window_name_position" => {
                let val = value.to_lowercase();
                if &val == "left" || &val == "right" || &val == "middle" {
                    self.window_name_position = value;
                } else {
                    return Err(format!("option parsing error: Option {name} takes one of these arguments: left, middle, right; {value} was supplied.").into());
                }
            }
            "display_name" => self.display_name = value,
            "gap_top" => {
                let val = value.to_lowercase().parse::<bool>()?;

                self.gap_top = val;
            }
            "gap_bottom" => {
                let val = value.to_lowercase().parse::<bool>()?;

                self.gap_bottom = val;
            }
            "gap_left" => {
                let val = value.to_lowercase().parse::<bool>()?;

                self.gap_left = val;
            }
            "gap_right" => {
                let val = value.to_lowercase().parse::<bool>()?;

                self.gap_right = val;
            }
            "gap_top_size" => {
                let val = value.to_lowercase().parse::<u32>()?;

                self.gap_top_size = val;
            }
            "gap_bottom_size" => {
                let val = value.to_lowercase().parse::<u32>()?;

                self.gap_bottom_size = val;
            }
            "gap_left_size" => {
                let val = value.to_lowercase().parse::<u32>()?;

                self.gap_left_size = val;
            }
            "gap_right_size" => {
                let val = value.to_lowercase().parse::<u32>()?;

                self.gap_right_size = val;
            }
            _ => return Err(format!("option parsing error: Unknown option {name}").into()),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_bool_parsing() {
        assert_eq!("FALSE".to_lowercase().parse::<bool>().is_ok(), true);
        assert_eq!("TruE".to_lowercase().parse::<bool>().is_ok(), true);
    }
}
