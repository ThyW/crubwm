use std::collections::VecDeque;
use std::fs::read_to_string;
use std::io::Write;

use crate::config::Config;
use crate::errors::Error;
use crate::WmResult;

/// The default config path is located in `~/.config/crubwm/config`
const CONFIG_PATH: &str = ".config/crubwm/config";

/// Command line argument parser.
///
/// Takes an input in the form of a vector of Strings and returns a Result with either the parsed
/// commands or an ArgumentParserError;
pub struct ArgumentParser;

#[derive(Debug, PartialEq, Eq)]
/// Enumeration of all possible and legal command types which can be created from command line
/// arguments.
pub enum CommandType {
    /// Takes no other arguments.
    ///
    /// If this command is passed, all other commands are ignored and a help message will be
    /// printed out.
    Help,
    /// Takes one argument.
    ///
    /// Ignores the default config path, instead uses the one passed as an argument.
    Config,
}

#[derive(Debug)]
/// Represents a single parsed legal command.
pub struct Command {
    cmd_type: CommandType,
    args: Option<Vec<String>>,
}

impl Command {
    /// Create a new command given its type and number of arguments.
    fn new(cmd_type: CommandType, num_args: usize) -> Self {
        if num_args == 0 {
            Self {
                cmd_type,
                args: None,
            }
        } else {
            Self {
                cmd_type,
                args: Some(Vec::with_capacity(num_args)),
            }
        }
    }

    /// Does the command have some subarguments?
    fn has_args(&self) -> bool {
        self.args.is_some()
    }

    /// Does the command have its required amount of arguments?
    ///
    /// Returns true even if the command takes no additional arguments.
    fn is_complete(&self) -> bool {
        if self.has_args() {
            let temp = self.args.as_ref().unwrap();
            temp.len() == temp.capacity()
        } else {
            true
        }
    }

    /// Attempt to add a new argument.
    ///
    /// Returns true if it's already full or if the addition filled it up.
    /// Returns false otherwise.
    fn add(&mut self, item: String) -> bool {
        if self.is_complete() {
            true
        } else {
            self.args.as_mut().unwrap().push(item);

            self.is_complete()
        }
    }

    pub fn get_type(&self) -> &CommandType {
        &self.cmd_type
    }
}

/// Config file parser.
pub struct ConfigParser;

impl ArgumentParser {
    /// Take a list of command line arguments and parse them into a list of commands.
    pub fn parse(mut input: VecDeque<String>) -> WmResult<Vec<Command>> {
        let mut return_vec = Vec::new();

        let mut command: Option<Command> = None;

        while !input.is_empty() {
            let current = input.pop_front().unwrap();

            if !current.starts_with('-') {
                if command.is_some() {
                    let inner = command.as_mut().unwrap();
                    inner.add(current);
                } // else ignore
            } else {
                if command.is_some() {
                    let inner = command.take().unwrap();
                    if !inner.is_complete() {
                        return Err("argument error: sub-arguments missing.".into());
                    } else {
                        return_vec.push(inner)
                    }
                }
                match &current[..] {
                    "-h" | "--help" => {
                        command = Some(Command::new(CommandType::Help, 0));
                    }
                    "--config" => command = Some(Command::new(CommandType::Config, 1)),
                    _ => (),
                }
            }
        }

        if let Some(inner) = command {
            if inner.is_complete() {
                return_vec.push(inner)
            } else {
                return Err("argument error: sub-arguments missing.".into());
            }
        }
        Ok(return_vec)
    }
}

impl ConfigParser {
    /// Parse a config file.
    ///
    /// Given a list of commands already received, check whether the `--config` command has been
    /// passed and read the new path, otherwise read the default config file which is located in
    /// `~/.config/crubwm/config`.
    pub fn parse(commands: &Vec<Command>) -> WmResult<Config> {
        let mut ret = Config::default();
        let mut default_path = std::env::var("HOME").map_err(|_| {
            Error::Generic("parsing error: unable to read $HOME environmental variable.".into())
        })?;
        default_path.push('/');
        default_path.push_str(CONFIG_PATH);
        let mut path = default_path.clone();

        // check whether a different config file should be loaded
        for command in commands {
            if command.get_type() == &CommandType::Config {
                path = command.args.as_ref().unwrap()[0].clone();
            } else if command.get_type() == &CommandType::Help {
                return Ok(ret);
            }
        }

        if !std::path::PathBuf::from(&default_path).exists() {
            let mut new_config_file = std::fs::File::create(default_path)?;

            new_config_file.write(ret.serialize()?)?;
        }

        let file_contents = read_to_string(path)?;

        for line in file_contents.lines() {
            if !line.is_empty() {
                let config_line = ConfigLine::try_from(line.to_owned())?;
                match config_line {
                    ConfigLine::Comment(..) => {}
                    ConfigLine::Keybind {
                        keys,
                        mut action,
                        action_arguments,
                    } => {
                        action.push(' ');
                        action.push_str(&action_arguments.join(" "));
                        ret.keybinds.add(keys, action)?
                    }
                    ConfigLine::Hook {
                        hook_type,
                        hook_args,
                        hook_option,
                    } => {
                        ret.start_hooks.add(hook_type, hook_args, hook_option)?;
                    }
                    ConfigLine::Option {
                        option_name,
                        option_value,
                    } => {
                        ret.options.add(option_name, option_value)?;
                    }
                    ConfigLine::WorkspaceSetting {
                        workspace_identifier,
                        workspace_setting_name,
                        workspace_setting_value,
                    } => {
                        ret.workspace_settings.add(
                            workspace_identifier.parse::<u32>()?,
                            workspace_setting_name,
                            workspace_setting_value,
                        )?;
                    }
                }
            }
        }

        Ok(ret)
    }
}

#[derive(Debug)]
/// A representation of a single line in a config file.
/// This is a representation of all the possible config lines.
enum ConfigLine {
    /// A comment line which starts with `#`
    Comment(String),
    /// A new keybind declaration
    Keybind {
        /// A string which represents one or multiple keys, to which we want to bind to
        keys: String,
        /// The action which we are binding
        action: String,
        /// Different arguments, depending on action.
        action_arguments: Vec<String>,
    },
    /// Representation of a hook.
    Hook {
        /// The type of the hook
        hook_type: String,
        /// Arguments of the hooks
        hook_args: Vec<String>,
        /// Type of the hook
        hook_option: String,
    },
    /// An option line which represents a single setting such as window border color
    Option {
        /// Name of the option
        option_name: String,
        /// Value of the option
        option_value: String,
    },
    WorkspaceSetting {
        workspace_identifier: String,
        workspace_setting_name: String,
        workspace_setting_value: Vec<String>,
    },
}

impl TryFrom<String> for ConfigLine {
    type Error = Error;

    fn try_from(line: String) -> WmResult<Self> {
        #[cfg(debug_assertions)]
        println!("[DEBUG] parsing config line: {line}");
        if let Some(s) = line.strip_prefix("keybind ") {
            let rest_of_line = s;
            let parser = LineParser::parse(rest_of_line.to_string());

            return Ok(Self::Keybind {
                keys: parser.0[0].clone(),
                action: parser.0[1].clone(),
                action_arguments: parser.0[2..].to_vec(),
            });
        } else if let Some(s) = line.strip_prefix("option ") {
            let rest_of_line = s;
            let parser = LineParser::parse(rest_of_line.to_string());

            return Ok(Self::Option {
                option_name: parser.0[0].clone(),
                option_value: parser.0[1].clone(),
            });
        } else if let Some(s) = line.strip_prefix("hook ") {
            let rest_of_line = s;
            let parser = LineParser::parse(rest_of_line.to_string());

            return Ok(Self::Hook {
                hook_type: parser.0[0].clone(),
                hook_option: parser.0[1].clone(),
                hook_args: parser.0[2..].to_vec(),
            });
        } else if let Some(s) = line.strip_prefix("workspace_setting") {
            let rest_of_line = s;
            let parser = LineParser::parse(rest_of_line.to_string());

            return Ok(Self::WorkspaceSetting {
                workspace_identifier: parser.0[0].clone(),
                workspace_setting_name: parser.0[1].clone(),
                workspace_setting_value: parser.0[2..].to_vec(),
            });
        } else if line.starts_with('#') {
            return Ok(Self::Comment(line));
        }

        #[cfg(debug_assertions)]
        println!("here, {line}");

        Err("config parsing error: unable to parse config file line".into())
    }
}

#[derive(Debug)]
/// Parse a single line of a file
struct LineParser(Vec<String>);

impl LineParser {
    fn parse(input: String) -> Self {
        let mut in_str = false;
        let mut in_escape = false;
        let mut buffer = String::new();
        let mut string_list = Vec::new();

        for current in input.chars() {
            if !in_str {
                if current == '"' {
                    in_str = true;
                } else if current == ' ' {
                    if buffer.is_empty() {
                        continue;
                    }
                    string_list.push(buffer.clone());
                    buffer.clear()
                } else {
                    buffer.push(current)
                }
            } else if current == '\\' {
                in_escape = true
            } else if in_escape && current == '"' {
                buffer.push(current);
                in_escape = false
            } else if current == '"' {
                in_str = false;
                string_list.push(buffer.clone());
                buffer.clear();
            } else {
                buffer.push(current)
            }
        }

        if !buffer.is_empty() {
            string_list.push(buffer)
        }
        Self(string_list)
    }
}
