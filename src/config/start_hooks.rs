use crate::errors::{Error, WmResult};

#[derive(Debug, Clone)]
pub enum HookType {
    Startup,
    Always,
}

impl TryFrom<String> for HookType {
    type Error = Error;

    fn try_from(value: String) -> WmResult<Self> {
        return match value.to_lowercase().as_ref() {
            "startup" => Ok(Self::Startup),
            "always" => Ok(Self::Always),
            _ => Err(format!("hook parsing error: Unable to parse hook type {value}").into()),
        };
    }
}

#[derive(Debug, Clone)]
pub struct Hook {
    pub hook_type: HookType,
    pub hook_args: Vec<String>,
}

impl Hook {
    fn new(hook_type: String, hook_args: Vec<String>) -> WmResult<Self> {
        let hook_type: HookType = hook_type.try_into()?;
        Ok(Self {
            hook_type,
            hook_args,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct StartHooks(Vec<Hook>);

impl StartHooks {
    pub fn add(&mut self, hook_type: String, hook_args: Vec<String>) -> WmResult {
        let hook = Hook::new(hook_type, hook_args)?;
        self.0.push(hook);

        Ok(())
    }

    pub fn run(&self) -> WmResult {
        for hook in &self.0 {
            let _ = std::process::Command::new("bash")
                .arg("-c")
                .args(hook.hook_args.as_slice())
                .spawn()?;
        }
        Ok(())
    }
}
