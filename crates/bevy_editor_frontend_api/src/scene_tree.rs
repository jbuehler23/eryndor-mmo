use bevy::prelude::*;

/// Lightweight view-model representing a single node in the editor scene tree.
#[derive(Clone, Debug)]
pub struct SceneTreeNode {
    pub entity: Entity,
    pub name: String,
    pub has_children: bool,
    pub children: Vec<Entity>,
}

impl SceneTreeNode {
    pub fn new(entity: Entity, name: String, has_children: bool, children: Vec<Entity>) -> Self {
        Self {
            entity,
            name,
            has_children,
            children,
        }
    }
}

/// Commands emitted by frontends to manipulate the scene tree.
#[derive(Event, Message, Debug, Clone)]
pub enum SceneTreeCommand {
    AddEntity {
        parent: Option<Entity>,
    },
    AddTemplateEntity {
        template: SceneEntityTemplate,
        parent: Option<Entity>,
    },
    DeleteEntity {
        entity: Entity,
    },
    RenameEntity {
        entity: Entity,
        new_name: String,
    },
    ReparentEntity {
        entity: Entity,
        new_parent: Option<Entity>,
    },
}

/// Identifiers for the built-in entity templates the editor understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneEntityTemplate {
    Empty,
    Sprite,
    Camera2D,
    UiNode,
    Button,
    Text,
}

impl SceneEntityTemplate {
    /// Convenience helper returning every known template.
    pub const ALL: [SceneEntityTemplate; 6] = [
        SceneEntityTemplate::Empty,
        SceneEntityTemplate::Sprite,
        SceneEntityTemplate::Camera2D,
        SceneEntityTemplate::UiNode,
        SceneEntityTemplate::Button,
        SceneEntityTemplate::Text,
    ];

    /// Human-readable label shown in menus.
    pub const fn display_name(self) -> &'static str {
        match self {
            SceneEntityTemplate::Empty => "Empty Entity",
            SceneEntityTemplate::Sprite => "Sprite",
            SceneEntityTemplate::Camera2D => "Camera 2D",
            SceneEntityTemplate::UiNode => "UI Node",
            SceneEntityTemplate::Button => "Button",
            SceneEntityTemplate::Text => "Text",
        }
    }

    /// Default entity name applied when the template is spawned.
    pub const fn default_name(self) -> &'static str {
        match self {
            SceneEntityTemplate::Empty => "New Entity",
            SceneEntityTemplate::Sprite => "Sprite",
            SceneEntityTemplate::Camera2D => "Camera",
            SceneEntityTemplate::UiNode => "UI Node",
            SceneEntityTemplate::Button => "Button",
            SceneEntityTemplate::Text => "Text",
        }
    }
}
