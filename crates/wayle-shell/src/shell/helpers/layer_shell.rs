use std::sync::Arc;

use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::gtk;
use tracing::warn;
use wayle_config::{ConfigService, schemas::general::Layer as ConfigLayer};
use wayle_hyprland::HyprlandService;
use wayle_niri::NiriService;

use super::monitors::current_monitors;

/// Clears all layer-shell edge anchors and zeroes margins.
pub(crate) fn reset_anchors(root: &gtk::Window) {
    root.set_anchor(Edge::Top, false);
    root.set_anchor(Edge::Bottom, false);
    root.set_anchor(Edge::Left, false);
    root.set_anchor(Edge::Right, false);

    root.set_margin(Edge::Top, 0);
    root.set_margin(Edge::Bottom, 0);
    root.set_margin(Edge::Left, 0);
    root.set_margin(Edge::Right, 0);
}

/// Maps a config [`ConfigLayer`] to a layer-shell [`Layer`].
pub(crate) fn to_gtk_layer(layer: ConfigLayer) -> Layer {
    match layer {
        ConfigLayer::Background => Layer::Background,
        ConfigLayer::Bottom => Layer::Bottom,
        ConfigLayer::Top => Layer::Top,
        ConfigLayer::Overlay => Layer::Overlay,
    }
}

/// Returns the layer to actually use after honoring `general.tearing-mode`.
///
/// Tearing-mode demotes `Overlay` to `Top` so fullscreen tearing works; other
/// layers pass through unchanged.
pub(crate) fn effective_layer(configured: ConfigLayer, tearing: bool) -> ConfigLayer {
    if tearing && configured == ConfigLayer::Overlay {
        ConfigLayer::Top
    } else {
        configured
    }
}

/// Applies the configured layer, honoring `general.tearing-mode`.
pub(crate) fn apply_layer(
    root: &gtk::Window,
    configured: ConfigLayer,
    config: &Arc<ConfigService>,
) {
    let tearing = config.config().general.tearing_mode.get();
    let layer = effective_layer(configured, tearing);
    root.set_layer(to_gtk_layer(layer));
}

/// Resolves and applies a monitor by connector name, falling back to primary.
pub(crate) fn apply_monitor_by_connector(root: &gtk::Window, connector: &str) {
    let monitors = current_monitors();
    let mut primary = None;
    let mut matched = None;

    for (name, monitor) in monitors {
        if primary.is_none() {
            primary = Some(monitor.clone());
        }

        if name == connector {
            matched = Some(monitor);
            break;
        }
    }

    if matched.is_none() {
        warn!(
            connector,
            "configured monitor not found, falling back to primary"
        );
    }

    root.set_monitor(matched.or(primary).as_ref());
}

/// Assigns the first available monitor to the layer-shell surface.
pub(crate) fn apply_primary_monitor(root: &gtk::Window) {
    let monitors = current_monitors();

    let primary = monitors.into_iter().next().map(|(_, monitor)| monitor);

    root.set_monitor(primary.as_ref());
}

/// Pins the surface to the monitor that currently has compositor focus.
///
/// Falls back to the primary monitor when the focused monitor can't be
/// resolved — e.g. no supported compositor is running, or the focused
/// connector doesn't match any GDK monitor.
pub(crate) fn apply_focused_monitor(
    root: &gtk::Window,
    hyprland: Option<&Arc<HyprlandService>>,
    niri: Option<&Arc<NiriService>>,
) {
    match focused_connector(hyprland, niri) {
        Some(connector) => apply_monitor_by_connector(root, &connector),
        None => {
            warn!("no focused monitor resolvable, falling back to primary");
            apply_primary_monitor(root);
        }
    }
}

/// Resolves the connector name of the focused monitor from whichever
/// compositor service is available.
///
/// Hyprland reports `focused` per monitor directly; niri's focused output is
/// the one hosting the focused workspace.
fn focused_connector(
    hyprland: Option<&Arc<HyprlandService>>,
    niri: Option<&Arc<NiriService>>,
) -> Option<String> {
    if let Some(hyprland) = hyprland {
        let focused = hyprland
            .monitors
            .get()
            .into_iter()
            .find(|monitor| monitor.focused.get())
            .map(|monitor| monitor.name.get());
        if focused.is_some() {
            return focused;
        }
    }

    if let Some(niri) = niri {
        return niri
            .workspaces
            .get()
            .values()
            .find(|workspace| workspace.is_focused.get())
            .and_then(|workspace| workspace.output.get());
    }

    None
}
