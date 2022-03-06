mod config;
mod errors;
mod parsers;
mod wm;

use errors::WmResult;
use parsers::{ArgumentParser, ConfigParser};
use wm::Wm;

use std::collections::VecDeque;

fn main() -> WmResult {
    let args: VecDeque<String> = std::env::args().collect();
    let commands = ArgumentParser::parse(args)?;
    let config = ConfigParser::parse(&commands)?;

    Wm::new(config)?.run(commands)
}
