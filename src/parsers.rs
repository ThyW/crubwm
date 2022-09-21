use std::fs::read_to_string;
use std::io::Write;

use hp::ParsedArguments;

use crate::config::Config;
use crate::errors::Error;
use crate::WmResult;

/// The default config path is located in `~/.config/crubwm/config`
const CONFIG_PATH: &str = ".config/crubwm/config";

/// Config file parser.
pub struct ConfigParser;

impl ConfigParser {
    /// Parse a config file.
    ///
    /// Given a list of commands already received, check whether the `--config` command has been
    /// passed and read the new path, otherwise read the default config file which is located in
    /// `~/.config/crubwm/config`.
    pub fn parse(
        commands: Option<&ParsedArguments>,
        path_arg: Option<&str>,
    ) -> WmResult<Config> {
        let mut ret = Config::default();
        let mut default_path = std::env::var("HOME").map_err(|_| {
            Error::Generic("parsing error: unable to read $HOME environmental variable.".into())
        })?;
        default_path.push('/');
        default_path.push_str(CONFIG_PATH);
        let mut path = default_path.clone();

        if let Some(arguments) = commands {
            if let Some(config_file) = arguments.get("--config") {
                path = config_file.values()[0].clone()
            }
        }

        if let Some(ppath) = path_arg {
            path = ppath.to_string()
        }

        if !std::path::PathBuf::from(&default_path).exists() {
            let mut new_config_file = std::fs::File::create(default_path)?;

            new_config_file.write_all(ret.serialize()?)?;
        }

        let file_contents = read_to_string(&path)?;

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
                    ConfigLine::Setting {
                        setting_name: option_name,
                        setting_value: option_value,
                    } => {
                        ret.settings.add(option_name, option_value)?;
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
                    ConfigLine::BarSetting {
                        bar_identifier,
                        bar_setting_name,
                        bar_setting_values,
                    } => {
                        ret.bar_settings.add(
                            bar_identifier.parse::<u32>()?,
                            bar_setting_name,
                            bar_setting_values,
                        )?;
                    }
                }
            }
        }

        ret.path = path;

        Ok(ret)
    }

    pub fn parse_with_path(path: &str) -> WmResult<Config> {
        Self::parse(None, Some(path))
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
    Setting {
        /// Name of the option
        setting_name: String,
        /// Value of the option
        setting_value: String,
    },
    WorkspaceSetting {
        workspace_identifier: String,
        workspace_setting_name: String,
        workspace_setting_value: Vec<String>,
    },
    BarSetting {
        bar_identifier: String,
        bar_setting_name: String,
        bar_setting_values: Vec<String>,
    },
}

impl TryFrom<String> for ConfigLine {
    type Error = Error;

    fn try_from(line: String) -> WmResult<Self> {
        if let Some(s) = line.strip_prefix("keybind ") {
            let rest_of_line = s;
            let parser = LineParser::parse(rest_of_line.to_string());

            return Ok(Self::Keybind {
                keys: parser.0[0].clone(),
                action: parser.0[1].clone(),
                action_arguments: parser.0[2..].to_vec(),
            });
        } else if let Some(s) = line.strip_prefix("set ") {
            let rest_of_line = s;
            let parser = LineParser::parse(rest_of_line.to_string());

            return Ok(Self::Setting {
                setting_name: parser.0[0].clone(),
                setting_value: parser.0[1].clone(),
            });
        } else if let Some(s) = line.strip_prefix("hook ") {
            let rest_of_line = s;
            let parser = LineParser::parse(rest_of_line.to_string());

            return Ok(Self::Hook {
                hook_type: parser.0[0].clone(),
                hook_option: parser.0[1].clone(),
                hook_args: parser.0[2..].to_vec(),
            });
        } else if let Some(s) = line.strip_prefix("workspace_set") {
            let rest_of_line = s;
            let parser = LineParser::parse(rest_of_line.to_string());

            return Ok(Self::WorkspaceSetting {
                workspace_identifier: parser.0[0].clone(),
                workspace_setting_name: parser.0[1].clone(),
                workspace_setting_value: parser.0[2..].to_vec(),
            });
        } else if let Some(s) = line.strip_prefix("bar_set ") {
            let rest_of_line = s;
            let parser = LineParser::parse(rest_of_line.to_string());

            return Ok(Self::BarSetting {
                bar_identifier: parser.0[0].clone(),
                bar_setting_name: parser.0[1].clone(),
                bar_setting_values: parser.0[2..].to_vec(),
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
