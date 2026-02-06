use bevy::prelude::*;
use bevy_editor_foundation::EditorTool;
use std::path::PathBuf;

/// High-level category describing how a frontend surfaces the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrontendKind {
    /// Graphical user interface (egui, wgpu, etc.).
    Gui,
    /// Terminal-based interactive frontend.
    Cli,
    /// Headless or automation-oriented frontend.
    Headless,
}

impl Default for FrontendKind {
    fn default() -> Self {
        FrontendKind::Gui
    }
}

/// Capability metadata advertised by a frontend implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrontendCapabilities {
    /// Whether this frontend expects CLI helpers (for running cargo, etc.).
    pub requires_cli: bool,
    /// Whether the frontend renders multiple viewports or windows.
    pub supports_multiple_viewports: bool,
}

impl Default for FrontendCapabilities {
    fn default() -> Self {
        Self {
            requires_cli: false,
            supports_multiple_viewports: false,
        }
    }
}

/// Trait implemented by modular editor frontends.
///
/// Frontends are responsible for installing any Bevy plugins and UI systems
/// required to render their interface. Backends interact with the frontend
/// through the [`EditorAction`] and [`EditorEvent`] channels.
pub trait EditorFrontend: Send + Sync + 'static {
    /// Stable identifier used for logging and configuration.
    fn id(&self) -> &'static str;

    /// Classify the frontend (gui/cli/headless).
    fn kind(&self) -> FrontendKind {
        FrontendKind::default()
    }

    /// Advertise optional capabilities so the shell can wire extras.
    fn capabilities(&self) -> FrontendCapabilities {
        FrontendCapabilities::default()
    }

    /// Install all systems, resources, and plugins necessary for this frontend.
    fn install(&self, app: &mut App);
}

/// Actions emitted by frontends to request work from backend crates.
#[derive(Event, Message, Debug, Clone, PartialEq)]
pub enum EditorAction {
    /// Request that the backend open a project (None = show dialog).
    RequestOpenProject { path: Option<PathBuf> },
    /// Ask the backend to create a new project using an optional template id.
    RequestCreateProject { template: Option<String> },
    /// Toggle the open project closed.
    RequestCloseProject,
    /// Request that the active scene is loaded from disk.
    RequestOpenScene { path: PathBuf },
    /// Ask the backend to save the active scene. None => Save As dialog.
    RequestSaveScene { path: Option<PathBuf> },
    /// Toggle a well-known editor panel.
    TogglePanel { panel: EditorPanel },
    /// Select a high-level editor tool.
    SelectTool(EditorTool),
    /// Update whether grid snapping is active.
    SetGridSnap { enabled: bool },
    /// Dispatch one of the built-in project commands (cargo run, etc.).
    RunProjectCommand { command: ProjectCommand },
    /// Request that any running project command is interrupted.
    CancelProjectCommand,
}

/// Notifications sent from backend crates to frontends.
#[derive(Event, Message, Debug, Clone, PartialEq, Eq)]
pub enum EditorEvent {
    ProjectOpened {
        path: PathBuf,
    },
    ProjectClosed,
    SceneLoaded {
        path: PathBuf,
    },
    SceneSaved {
        path: PathBuf,
    },
    ProjectCommandStarted {
        command: ProjectCommand,
    },
    ProjectCommandFinished {
        command: ProjectCommand,
        success: bool,
        message: Option<String>,
    },
    Error {
        message: String,
    },
}

/// Named panels that can be toggled via the shared frontend API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorPanel {
    Inspector,
    AssetBrowser,
    ProjectBrowser,
    Tileset,
    Layers,
}

/// Built-in project/CLI commands exposed to frontends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProjectCommand {
    Run,
    RunScene,
    RunWeb,
    Build,
    Lint,
}
