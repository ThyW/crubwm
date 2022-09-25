//! # About
//! Crubwm is a tiling window manager for the X11 windowing protocol. It is a hobby project and not
//! really meant to be a serious competitor, nor an alternative to any of the existing and
//! established window managers. The source code, as well as the configuration manual can be found
//! on [github](https://github.com/ThyW/crubwm).
//!
//! # Overview
//! As (mostly) every other Rust program, the window manager starts in the main function. We first
//! parse the command line arguments. After that the config is loaded and parsed, the logger is
//! initialized. After that, the main event processing loop is started.
//!
//! - for an overview of the command line and config file parsers, have a look into [`parsers`][crate::parsers].
//! - for an overview of the configuration settings, have a look at [`config`][crate::config].
//! - when looking for the actual window managing implementation, take a look at the [`wm module`][crate::wm]
//! and the [`State`][crate::wm::state::State] struct.

/// All the configuration settings and defaults.
pub mod config;
/// WmResult and Error types.
pub mod errors;
/// C types and helper functions.
pub mod ffi;
/// Info and error logging utilities.
pub mod log;
/// Implementation of the command line option and config file parsers.
pub mod parsers;
/// General utilities.
pub mod utils;
/// Window manager implementation and utilities.
pub mod wm;

use errors::WmResult;
use hp::{Parser, Template};
use log::prepare_logger;
use parsers::ConfigParser;
use wm::Wm;

use std::{fmt::Display, process::exit};

fn main() {
    let mut parser = Parser::new()
        .with_author("zir")
        .with_description("tiling x11 window manager.")
        .with_program_name("crubwm")
        .exit_on_help(true);
    parser.add_template(
        Template::new()
            .matches("-c")
            .matches("--config")
            .with_help("Specify a config to run with")
            .number_of_values(1)
            .optional_values(false),
    );

    let command_line_arguments_res = parser.parse(None);

    if let Ok(command_line_arguments) = print_err(command_line_arguments_res) {
        if let Ok(config) = print_err(ConfigParser::parse(Some(&command_line_arguments), None)) {
            if print_err(prepare_logger(
                &config.settings.log_file,
                config.settings.log_level,
            ))
            .is_ok()
            {
                if let Ok(mut wm) = print_err(Wm::new(config)) {
                    if print_err(wm.run()).is_err() {
                        exit(1)
                    }
                }
            }
        }
    }
}

fn print_err<T, E: Into<errors::Error> + Display>(input: Result<T, E>) -> WmResult<T> {
    match input {
        Ok(t) => Ok(t),
        Err(e) => {
            eprintln!("{}", &e);
            Err(e.into())
        }
    }
}
