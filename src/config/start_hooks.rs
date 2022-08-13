use crate::config::Repr;
use crate::errors::{Error, WmResult};

#[derive(Debug, Clone)]
pub enum HookType {
    Startup,
    Always,
    After,
}

#[derive(Debug, Clone)]
pub enum HookOption {
    Sync,
    Async,
}

impl TryFrom<String> for HookType {
    type Error = Error;

    fn try_from(value: String) -> WmResult<Self> {
        return match value.to_lowercase().as_ref() {
            "startup" => Ok(Self::Startup),
            "always" => Ok(Self::Always),
            "after" => Ok(Self::After),
            _ => Err(format!("hook parsing error: Unable to parse hook type {value}").into()),
        };
    }
}

impl TryFrom<String> for HookOption {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        return match value.to_lowercase().as_str() {
            "async" => Ok(Self::Async),
            "sync" => Ok(Self::Sync),
            _ => Err(format!("hook parsing error: Unable to parse hook option {value}").into()),
        };
    }
}

#[derive(Debug, Clone)]
pub struct Hook {
    pub hook_type: HookType,
    pub hook_option: HookOption,
    pub hook_args: Vec<String>,
}

impl Hook {
    fn new(hook_type: String, hook_args: Vec<String>, hook_option: String) -> WmResult<Self> {
        let hook_type: HookType = hook_type.try_into()?;
        let hook_option: HookOption = hook_option.try_into()?;
        Ok(Self {
            hook_type,
            hook_args,
            hook_option,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct StartHooks(Vec<Hook>);

impl StartHooks {
    pub fn add(
        &mut self,
        hook_type: String,
        hook_args: Vec<String>,
        hook_option: String,
    ) -> WmResult {
        let hook = Hook::new(hook_type, hook_args, hook_option)?;
        self.0.push(hook);

        Ok(())
    }

    pub fn run(&self) -> WmResult {
        for hook in &self.0 {
            match hook.hook_option {
                HookOption::Sync => {
                    let _ = std::process::Command::new("bash")
                        .arg("-c")
                        .args(hook.hook_args.as_slice())
                        .spawn()?
                        .wait()?;
                }
                HookOption::Async => {
                    let _ = std::process::Command::new("bash")
                        .arg("-c")
                        .args(hook.hook_args.as_slice())
                        .spawn()?;
                }
            }
        }
        Ok(())
    }

    pub fn run_after(&self) -> WmResult {
        for hook in self.0.iter() {
            if let HookType::After = hook.hook_type {
                match hook.hook_option {
                    HookOption::Sync => {
                        let _ = std::process::Command::new("bash")
                            .arg("-c")
                            .args(hook.hook_args.as_slice())
                            .spawn()?
                            .wait()?;
                    }
                    HookOption::Async => {
                        let _ = std::process::Command::new("bash")
                            .arg("-c")
                            .args(hook.hook_args.as_slice())
                            .spawn()?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl Repr for StartHooks {
    fn repr(&self) -> WmResult<String> {
        Ok("ahoy".to_string())
    }
}
