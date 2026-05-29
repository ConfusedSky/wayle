use std::sync::Arc;

use wayle_hyprland::HyprlandService;
use wayle_niri::NiriService;

/// Resolves the connector name of the monitor that should be treated as
/// focused.
pub(crate) trait FocusedMonitor {
    /// Connector name of the focused monitor, or `None` when it can't be
    /// resolved right now (the caller falls back to the primary monitor).
    fn focused_connector(&self) -> Option<String>;
}

impl FocusedMonitor for HyprlandService {
    fn focused_connector(&self) -> Option<String> {
        self.monitors
            .get()
            .into_iter()
            .find(|monitor| monitor.focused.get())
            .map(|monitor| monitor.name.get())
    }
}

impl FocusedMonitor for NiriService {
    fn focused_connector(&self) -> Option<String> {
        self.workspaces
            .get()
            .values()
            .find(|workspace| workspace.is_focused.get())
            .and_then(|workspace| workspace.output.get())
    }
}

pub(crate) fn from_services(
    hyprland: Option<Arc<HyprlandService>>,
    niri: Option<Arc<NiriService>>,
) -> Option<Arc<dyn FocusedMonitor>> {
    hyprland
        .map(|hyprland| hyprland as Arc<dyn FocusedMonitor>)
        .or_else(|| niri.map(|niri| niri as Arc<dyn FocusedMonitor>))
}
