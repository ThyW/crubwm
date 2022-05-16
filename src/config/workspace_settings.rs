use crate::errors::{Error, WmResult};

#[derive(Clone, Debug)]
pub struct WorkspaceSettings {
    pub identifier: u32,
    pub name: String,
    pub allowed_layouts: Vec<String>,
    pub output: String,
    pub default_container_type: String,
}

impl WorkspaceSettings {
    pub fn new(identifier: u32) -> Self {
        Self {
            identifier,
            name: format!("{}", identifier),
            output: "".to_string(),
            allowed_layouts: vec!["all".to_string()],
            default_container_type: "tile".to_string(),
        }
    }

    pub fn with_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn with_allowed_layouts(&mut self, allowed_layouts: Vec<String>) {
        self.allowed_layouts = allowed_layouts;
    }

    pub fn with_output(&mut self, output: String) {
        self.output = output;
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
            "output" => {
                workspace.with_output(value[0].clone());
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

    pub fn get(&self) -> &Vec<WorkspaceSettings> {
        &self.0
    }
}

impl IntoIterator for AllWorkspaceSettings {
    type Item = WorkspaceSettings;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
