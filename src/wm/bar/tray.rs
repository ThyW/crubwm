use crate::config::IconTraySettings;

#[derive(Clone, Debug)]
pub struct IconTraySegment {
    icons: Vec<u32>,
    settings: IconTraySettings,
}

impl From<IconTraySettings> for IconTraySegment {
    fn from(s: IconTraySettings) -> Self {
        Self {
            icons: Vec::new(),
            settings: s,
        }
    }
}
