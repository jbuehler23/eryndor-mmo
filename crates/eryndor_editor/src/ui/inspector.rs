use bevy_egui::egui;

use crate::project::Project;
use crate::schema::{render_instance_form, FormContext, FormResult};
use crate::EditorState;

/// Result of inspector actions that need to be handled by the caller
#[derive(Default)]
pub struct InspectorResult {
    pub delete_data_instance: Option<uuid::Uuid>,
    pub delete_entity: Option<(uuid::Uuid, uuid::Uuid)>, // (level_id, entity_id)
    /// Sprite editor to open: (property_name, instance_id)
    pub open_sprite_editor: Option<(String, uuid::Uuid)>,
    /// Dialogue editor to open: (property_name, instance_id)
    pub open_dialogue_editor: Option<(String, uuid::Uuid)>,
}

pub fn render_inspector(ui: &mut egui::Ui, editor_state: &mut EditorState, project: &mut Project) -> InspectorResult {
    let mut result = InspectorResult::default();

    ui.heading("Inspector");
    ui.separator();

    match &editor_state.selection {
        Selection::None => {
            ui.label("Nothing selected");
            ui.label("");
            ui.label("Select an item from the tree view");
            ui.label("or an entity from the viewport");
        }
        Selection::DataInstance(id) => {
            let (delete, open_sprite, open_dialogue) = render_data_instance_inspector(ui, *id, project);
            result.delete_data_instance = delete;
            result.open_sprite_editor = open_sprite;
            result.open_dialogue_editor = open_dialogue;
        }
        Selection::EntityInstance(level_id, entity_id) => {
            result.delete_entity = render_entity_instance_inspector(ui, *level_id, *entity_id, project);
        }
        Selection::Level(id) => {
            render_level_inspector(ui, *id, project);
        }
        Selection::Layer(level_id, layer_index) => {
            render_layer_inspector(ui, *level_id, *layer_index, project);
        }
    }

    result
}

/// Returns (delete_instance_id, open_sprite_editor, open_dialogue_editor)
fn render_data_instance_inspector(
    ui: &mut egui::Ui,
    id: uuid::Uuid,
    project: &mut Project,
) -> (Option<uuid::Uuid>, Option<(String, uuid::Uuid)>, Option<(String, uuid::Uuid)>) {
    let mut delete_requested = false;
    let mut form_result = FormResult::default();

    // Get info we need from schema first (immutable borrow)
    let (type_name, type_def, schema_clone) = {
        let instance = match project.data.get(id) {
            Some(i) => i,
            None => {
                ui.label("Instance not found");
                return (None, None, None);
            }
        };
        let type_name = instance.type_name.clone();
        let type_def = project.schema.get_type(&type_name).cloned();
        let schema_clone = project.schema.clone();
        (type_name, type_def, schema_clone)
    };

    // Build all_instances map for FormContext (for Ref dropdowns)
    let all_instances = project.data.instances.clone();

    // Now we can get a mutable reference
    let instance = match project.data.get_mut(id) {
        Some(i) => i,
        None => return (None, None, None),
    };

    // Header with type name and ID
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(&type_name).heading());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("🗑").on_hover_text("Delete").clicked() {
                delete_requested = true;
            }
        });
    });
    ui.label(format!("ID: {}", &instance.id.to_string()[..8]));
    ui.separator();

    // Use the full form generator if we have a type definition
    if let Some(type_def) = &type_def {
        let ctx = FormContext {
            schema: &schema_clone,
            all_instances: &all_instances,
        };

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                form_result = render_instance_form(ui, &schema_clone, type_def, instance, &ctx);
            });
    } else {
        ui.label(format!("Unknown type: {}", type_name));
    }

    if form_result.changed {
        project.mark_dirty();
    }

    let delete = if delete_requested { Some(id) } else { None };
    (delete, form_result.open_sprite_editor, form_result.open_dialogue_editor)
}

fn render_entity_instance_inspector(
    ui: &mut egui::Ui,
    level_id: uuid::Uuid,
    entity_id: uuid::Uuid,
    project: &mut Project,
) -> Option<(uuid::Uuid, uuid::Uuid)> {
    let mut delete_requested = false;

    // Get type info and clone schema first (immutable borrow)
    let (type_name, type_def, schema_clone) = {
        let level = match project.get_level(level_id) {
            Some(l) => l,
            None => {
                ui.label("Level not found");
                return None;
            }
        };
        let entity = match level.get_entity(entity_id) {
            Some(e) => e,
            None => {
                ui.label("Entity not found");
                return None;
            }
        };
        let type_name = entity.type_name.clone();
        let type_def = project.schema.get_type(&type_name).cloned();
        let schema_clone = project.schema.clone();
        (type_name, type_def, schema_clone)
    };

    // Now get mutable reference
    let level = match project.get_level_mut(level_id) {
        Some(l) => l,
        None => return None,
    };
    let entity = match level.get_entity_mut(entity_id) {
        Some(e) => e,
        None => return None,
    };

    // Header with type name and delete button
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(&type_name).heading());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("🗑").on_hover_text("Delete").clicked() {
                delete_requested = true;
            }
        });
    });
    ui.label(format!("ID: {}", &entity.id.to_string()[..8]));
    ui.separator();

    // Position
    ui.horizontal(|ui| {
        ui.label("Position:");
        ui.add(egui::DragValue::new(&mut entity.position.x).prefix("x: "));
        ui.add(egui::DragValue::new(&mut entity.position.y).prefix("y: "));
    });

    ui.separator();

    // Properties (editable)
    if let Some(type_def) = &type_def {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for prop in &type_def.properties {
                    ui.horizontal(|ui| {
                        let label = if prop.required {
                            format!("{}*", prop.name)
                        } else {
                            prop.name.clone()
                        };
                        ui.label(&label);

                        match prop.prop_type {
                            crate::schema::PropType::String => {
                                let mut value = entity.properties
                                    .get(&prop.name)
                                    .and_then(|v| v.as_string())
                                    .unwrap_or("")
                                    .to_string();
                                if ui.text_edit_singleline(&mut value).changed() {
                                    entity.properties.insert(prop.name.clone(), crate::schema::Value::String(value));
                                }
                            }
                            crate::schema::PropType::Int => {
                                let mut value = entity.properties
                                    .get(&prop.name)
                                    .and_then(|v| v.as_int())
                                    .unwrap_or(0);
                                if ui.add(egui::DragValue::new(&mut value)).changed() {
                                    entity.properties.insert(prop.name.clone(), crate::schema::Value::Int(value));
                                }
                            }
                            crate::schema::PropType::Float => {
                                let mut value = entity.properties
                                    .get(&prop.name)
                                    .and_then(|v| v.as_float())
                                    .unwrap_or(0.0);
                                if ui.add(egui::DragValue::new(&mut value).speed(0.1)).changed() {
                                    entity.properties.insert(prop.name.clone(), crate::schema::Value::Float(value));
                                }
                            }
                            crate::schema::PropType::Bool => {
                                let mut value = entity.properties
                                    .get(&prop.name)
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                if ui.checkbox(&mut value, "").changed() {
                                    entity.properties.insert(prop.name.clone(), crate::schema::Value::Bool(value));
                                }
                            }
                            crate::schema::PropType::Enum => {
                                if let Some(enum_name) = &prop.enum_type {
                                    if let Some(enum_values) = schema_clone.get_enum(enum_name) {
                                        let current = entity.properties
                                            .get(&prop.name)
                                            .and_then(|v| v.as_string())
                                            .unwrap_or("")
                                            .to_string();
                                        let mut selected = current.clone();

                                        egui::ComboBox::from_id_salt(format!("entity_{}", prop.name))
                                            .selected_text(&selected)
                                            .show_ui(ui, |ui| {
                                                for value in enum_values {
                                                    ui.selectable_value(&mut selected, value.clone(), value);
                                                }
                                            });

                                        if selected != current {
                                            entity.properties.insert(prop.name.clone(), crate::schema::Value::String(selected));
                                        }
                                    }
                                }
                            }
                            _ => {
                                ui.label(format!("({:?})", prop.prop_type));
                            }
                        }
                    });
                }
            });
    }

    if delete_requested {
        Some((level_id, entity_id))
    } else {
        None
    }
}

fn render_level_inspector(ui: &mut egui::Ui, id: uuid::Uuid, project: &mut Project) {
    let level = match project.get_level_mut(id) {
        Some(l) => l,
        None => {
            ui.label("Level not found");
            return;
        }
    };

    ui.label(egui::RichText::new("Level").heading());
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut level.name);
    });

    ui.horizontal(|ui| {
        ui.label("Size:");
        ui.label(format!("{}x{}", level.width, level.height));
    });

    ui.separator();

    ui.label(format!("Layers: {}", level.layers.len()));
    ui.label(format!("Entities: {}", level.entities.len()));
}

fn render_layer_inspector(
    ui: &mut egui::Ui,
    level_id: uuid::Uuid,
    layer_index: usize,
    project: &mut Project,
) {
    let level = match project.get_level_mut(level_id) {
        Some(l) => l,
        None => {
            ui.label("Level not found");
            return;
        }
    };

    let layer = match level.layers.get_mut(layer_index) {
        Some(l) => l,
        None => {
            ui.label("Layer not found");
            return;
        }
    };

    ui.label(egui::RichText::new("Layer").heading());
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut layer.name);
    });

    ui.checkbox(&mut layer.visible, "Visible");

    ui.separator();

    ui.label(format!("Type: {:?}", layer.layer_type()));
}

#[derive(Debug, Clone, Default)]
pub enum Selection {
    #[default]
    None,
    DataInstance(uuid::Uuid),
    EntityInstance(uuid::Uuid, uuid::Uuid), // (level_id, entity_id)
    Level(uuid::Uuid),
    Layer(uuid::Uuid, usize), // (level_id, layer_index)
}

impl Selection {
    pub fn is_selected_data(&self, id: uuid::Uuid) -> bool {
        matches!(self, Selection::DataInstance(sel_id) if *sel_id == id)
    }

    pub fn is_selected_entity(&self, level_id: uuid::Uuid, entity_id: uuid::Uuid) -> bool {
        matches!(self, Selection::EntityInstance(l, e) if *l == level_id && *e == entity_id)
    }

    pub fn is_selected_level(&self, id: uuid::Uuid) -> bool {
        matches!(self, Selection::Level(sel_id) if *sel_id == id)
    }

    pub fn is_selected_layer(&self, level_id: uuid::Uuid, layer_index: usize) -> bool {
        matches!(self, Selection::Layer(l, i) if *l == level_id && *i == layer_index)
    }
}
