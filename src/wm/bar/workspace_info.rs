use cairo::Context;

use crate::{
    config::{WmResult, WorkspaceSegmentSettings},
    wm::{geometry::TextExtents, workspace::WorkspaceId},
};

/// The workspace info segment informs the user about the current state of the window manager's
/// workspaces. It shows information such as the workspaces available for the current monitor,
/// the focused workspace, workspace names and urgent workspaces.
#[derive(Debug, Clone)]
pub struct WorkspaceInfoSegment {
    /// Name of the workspace/what is displayed.
    name: String,
    /// Workspace number or id.
    workspace_id: WorkspaceId,
    /// Is the workspace focused?
    focused: bool,
    /// Is the workspace currently open?
    open: bool,
    /// Does the workspace seek urgent attention?
    urgent: bool,
}

/// The workspace info consists of different workspace info segments.
#[derive(Clone, Debug)]
pub struct WorkspaceInfo {
    workspaces: Vec<WorkspaceInfoSegment>,
    settings: WorkspaceSegmentSettings,
}

impl From<WorkspaceSegmentSettings> for WorkspaceInfo {
    fn from(s: WorkspaceSegmentSettings) -> Self {
        Self {
            workspaces: Vec::new(),
            settings: s,
        }
    }
}

impl WorkspaceInfoSegment {
    pub fn new(name: impl AsRef<str>, id: impl Into<u32>) -> Self {
        Self {
            name: name.as_ref().to_string(),
            workspace_id: id.into(),
            focused: false,
            open: false,
            urgent: false,
        }
    }

    fn value(&self) -> String {
        format!(" {}:{} ", self.name, self.workspace_id)
    }

    fn draw(&self, cr: &Context, settings: &WorkspaceSegmentSettings) -> WmResult {
        cr.select_font_face(
            &settings.font,
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal,
        );
        let text = self.value();

        if self.focused {
            let (r, g, b) = WorkspaceSegmentSettings::translate_color(
                settings.focused_foreground_color.clone(),
            )?;
            cr.set_source_rgb(r, g, b);
        } else {
            let (r, g, b) = WorkspaceSegmentSettings::translate_color(
                settings.normal_foreground_color.clone(),
            )?;
            cr.set_source_rgb(r, g, b);
        }

        cr.show_text(&text)?;

        Ok(())
    }

    fn get_extents(
        &self,
        cr: &Context,
        font_size: f64,
        settings: &WorkspaceSegmentSettings,
    ) -> WmResult<TextExtents> {
        cr.select_font_face(
            &settings.font,
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal,
        );
        cr.set_font_size(font_size);
        let ext = cr.text_extents(&self.value())?.into();

        Ok(ext)
    }
}

impl WorkspaceInfo {
    pub fn add(&mut self, input: WorkspaceInfoSegment) {
        self.workspaces.push(input)
    }

    pub fn set_focused(&mut self, ws: Option<WorkspaceId>) -> WmResult {
        if let Some(workspace_id) = ws {
            for segment in self.workspaces.iter_mut() {
                if segment.workspace_id == workspace_id {
                    segment.open = true;
                    segment.focused = true;
                } else {
                    segment.open = false;
                    segment.focused = false;
                }
            }
        }

        Ok(())
    }

    pub fn set_open(&mut self, ws: Option<WorkspaceId>) -> WmResult {
        if let Some(workspace_id) = ws {
            for segment in self.workspaces.iter_mut() {
                if segment.workspace_id == workspace_id {
                    segment.open = true;
                } else {
                    segment.open = false
                }
            }
        }
        Ok(())
    }

    pub fn get_text(&self) -> String {
        let mut buffer = String::new();

        for workspace in self.workspaces.iter() {
            buffer.push_str(&workspace.value())
        }

        buffer
    }

    pub fn get_text_extents(&self, cr: &Context, font_size: f64) -> WmResult<TextExtents> {
        let mut extents = TextExtents::default();

        for workspace in self.workspaces.iter() {
            extents += workspace.get_extents(cr, font_size, &self.settings)?;
        }

        Ok(extents)
    }

    pub fn draw(&self, cr: &Context, position: Option<(f32, f32)>) -> WmResult {
        if let Some((x, y)) = position {
            cr.move_to(x.into(), y.into());
        }
        for part in self.workspaces.iter() {
            part.draw(&cr, &self.settings)?
        }

        Ok(())
    }
}
