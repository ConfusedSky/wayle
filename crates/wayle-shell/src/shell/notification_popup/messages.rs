use std::sync::Arc;

use wayle_config::ConfigService;
use wayle_hyprland::HyprlandService;
use wayle_niri::NiriService;
use wayle_notification::{NotificationService, core::notification::Notification};

/// Initialization data for the notification popup host.
pub(crate) struct PopupHostInit {
    pub(crate) notification: Arc<NotificationService>,
    pub(crate) config: Arc<ConfigService>,
    pub(crate) hyprland: Option<Arc<HyprlandService>>,
    pub(crate) niri: Option<Arc<NiriService>>,
}

/// Commands for popup host updates.
#[derive(Debug)]
pub(crate) enum PopupHostCmd {
    PopupsChanged(Vec<Arc<Notification>>),
    ConfigChanged,
}
