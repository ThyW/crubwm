use crate::errors::{Error, WmResult};

use super::Repr;

#[derive(Clone, Debug)]
pub struct WorkspaceSettings {
    pub identifier: u32,
    pub name: String,
    pub allowed_layouts: Vec<String>,
    pub monitor: String,
    pub default_container_type: String,
}

impl WorkspaceSettings {
    pub fn new(identifier: u32) -> Self {
        Self {
            identifier,
            name: format!("{}", identifier),
            monitor: "".to_string(),
            allowed_layouts: vec!["all".to_string()],
            default_container_type: "in_layout".to_string(),
        }
    }

    pub fn with_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn with_allowed_layouts(&mut self, allowed_layouts: Vec<String>) {
        self.allowed_layouts = allowed_layouts;
    }

    pub fn with_monitor(&mut self, output: String) {
        self.monitor = output;
    }

    pub fn with_default_container_type(&mut self, default_container_type: String) {
        self.default_container_type = default_container_type;
    }
}

#[derive(Debug, Clone)]
pub struct AllWorkspaceSettings(Vec<WorkspaceSettings>);

impl Default for AllWorkspaceSettings {
    fn default() -> Self {
        let mut ret = vec![];

        for i in 1..11 {
            ret.push(WorkspaceSettings::new(i))
        }

        Self(ret)
    }
}

impl Repr for AllWorkspaceSettings {
    fn repr(&self) -> WmResult<String> {
        Ok("self".to_string())
    }
}

impl AllWorkspaceSettings {
    pub fn add(&mut self, identifier: u32, name: String, value: Vec<String>) -> WmResult {
        let workspace = self
            .0
            .iter_mut()
            .find(|w| w.identifier == identifier)
            .ok_or_else(|| {
                Error::Generic(format!(
                    "workspace setting parsing error: no workspace with identifier {identifier} found!"
                ))
            })?;
        match &name[..] {
            "name" => {
                workspace.with_name(value[0].clone());
            }
            "monitor" => {
                workspace.with_monitor(value[0].clone());
            }
            "allowed_layouts" => workspace.with_allowed_layouts(value),
            "default_container_type" => {
                workspace.with_default_container_type(value[0].clone());
            }
            _ => {
                return Err(format!(
                    "workspace setting parsing error: setting {name} does not exist!"
                )
                .into())
            }
        }
        Ok(())
    }
}

impl IntoIterator for AllWorkspaceSettings {
    type Item = WorkspaceSettings;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
