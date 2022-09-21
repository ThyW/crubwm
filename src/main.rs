#![feature(iter_collect_into)]
mod config;
mod errors;
mod ffi;
mod log;
mod parsers;
mod utils;
mod wm;

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
