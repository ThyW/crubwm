mod config;
mod errors;
mod parsers;
mod wm;

use errors::WmResult;
use parsers::{ArgumentParser, ConfigParser};
use wm::Wm;

use std::{collections::VecDeque, process::exit};

fn main() {
    #[cfg(debug_assertions)]
    println!("{:?}", (1, true, false));
    let args: VecDeque<String> = std::env::args().collect();
    if let Ok(commands) = print_err(ArgumentParser::parse(args)) {
        if let Ok(config) = print_err(ConfigParser::parse(&commands)) {
            if let Ok(mut wm) = print_err(Wm::new(config)) {
                if let Err(_) = print_err(wm.run(commands)) {
                    exit(1)
                }
            }
        }
    }
}

fn print_err<T>(input: WmResult<T>) -> WmResult<T> {
    match input {
        Ok(t) => Ok(t),
        Err(e) => {
            eprintln!("{}", &e);
            Err(e)
        }
    }
}
