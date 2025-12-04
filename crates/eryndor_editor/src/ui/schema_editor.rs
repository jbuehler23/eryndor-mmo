use bevy_egui::egui;

use crate::project::Project;
use crate::schema::{PropType, PropertyDef, TypeDef};

/// State for the schema editor dialog
#[derive(Default)]
pub struct SchemaEditorState {
    /// Which tab is selected: "Enums", "Data Types", "Embedded Types"
    pub selected_tab: SchemaTab,

    /// Currently selected enum name
    pub selected_enum: Option<String>,
    /// Currently selected type name
    pub selected_type: Option<String>,

    /// New enum name input
    pub new_enum_name: String,
    /// New enum value input
    pub new_enum_value: String,

    /// New type name input
    pub new_type_name: String,

    /// New property state
    pub new_property: NewPropertyState,

    /// Editing property (index and state)
    pub editing_property: Option<(usize, NewPropertyState)>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum SchemaTab {
    #[default]
    Enums,
    DataTypes,
    EmbeddedTypes,
}

impl SchemaTab {
    pub fn label(&self) -> &'static str {
        match self {
            SchemaTab::Enums => "Enums",
            SchemaTab::DataTypes => "Data Types",
            SchemaTab::EmbeddedTypes => "Embedded",
        }
    }
}

#[derive(Default, Clone)]
pub struct NewPropertyState {
    pub name: String,
    pub prop_type: PropType,
    pub required: bool,
    pub enum_type: String,
    pub ref_type: String,
    pub embedded_type: String,
    pub item_type: String,  // For arrays: "ref", "string", etc.
    pub min: String,
    pub max: String,
    pub show_if: String,
}

impl Default for PropType {
    fn default() -> Self {
        PropType::String
    }
}

impl NewPropertyState {
    pub fn to_property_def(&self) -> PropertyDef {
        PropertyDef {
            name: self.name.clone(),
            prop_type: self.prop_type,
            required: self.required,
            default: None,
            min: self.min.parse().ok(),
            max: self.max.parse().ok(),
            show_if: if self.show_if.is_empty() { None } else { Some(self.show_if.clone()) },
            enum_type: if self.enum_type.is_empty() { None } else { Some(self.enum_type.clone()) },
            ref_type: if self.ref_type.is_empty() { None } else { Some(self.ref_type.clone()) },
            item_type: if self.item_type.is_empty() { None } else { Some(self.item_type.clone()) },
            embedded_type: if self.embedded_type.is_empty() { None } else { Some(self.embedded_type.clone()) },
        }
    }

    pub fn from_property_def(prop: &PropertyDef) -> Self {
        Self {
            name: prop.name.clone(),
            prop_type: prop.prop_type,
            required: prop.required,
            enum_type: prop.enum_type.clone().unwrap_or_default(),
            ref_type: prop.ref_type.clone().unwrap_or_default(),
            embedded_type: prop.embedded_type.clone().unwrap_or_default(),
            item_type: prop.item_type.clone().unwrap_or_default(),
            min: prop.min.map(|v| v.to_string()).unwrap_or_default(),
            max: prop.max.map(|v| v.to_string()).unwrap_or_default(),
            show_if: prop.show_if.clone().unwrap_or_default(),
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// Render the schema editor dialog
pub fn render_schema_editor(
    ctx: &egui::Context,
    state: &mut SchemaEditorState,
    project: &mut Project,
    show: &mut bool,
) {
    egui::Window::new("Schema Editor")
        .open(show)
        .resizable(true)
        .default_width(450.0)
        .default_height(600.0)
        .vscroll(false)
        .show(ctx, |ui| {
            // Tab bar at top
            ui.horizontal(|ui| {
                for tab in [SchemaTab::Enums, SchemaTab::DataTypes, SchemaTab::EmbeddedTypes] {
                    if ui.selectable_label(state.selected_tab == tab, tab.label()).clicked() {
                        state.selected_tab = tab;
                        state.selected_enum = None;
                        state.selected_type = None;
                    }
                }
            });

            ui.separator();

            // Main content area with vertical scroll
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());

                    match state.selected_tab {
                        SchemaTab::Enums => render_enums_tab(ui, state, project),
                        SchemaTab::DataTypes => render_types_tab(ui, state, project, TypeCategory::Data),
                        SchemaTab::EmbeddedTypes => render_types_tab(ui, state, project, TypeCategory::Embedded),
                    }
                });
        });
}

fn render_enums_tab(ui: &mut egui::Ui, state: &mut SchemaEditorState, project: &mut Project) {
    let mut changed = false;
    let mut delete_enum: Option<String> = None;
    let mut to_remove: Option<usize> = None;
    let mut add_value: Option<String> = None;

    // Enum selector dropdown
    ui.horizontal(|ui| {
        ui.label("Enum:");

        let current_label = state.selected_enum.as_deref().unwrap_or("(select)");
        egui::ComboBox::from_id_salt("enum_selector")
            .selected_text(current_label)
            .show_ui(ui, |ui| {
                let mut enum_names: Vec<_> = project.schema.enums.keys().cloned().collect();
                enum_names.sort();
                for enum_name in &enum_names {
                    if ui.selectable_label(state.selected_enum.as_ref() == Some(enum_name), enum_name).clicked() {
                        state.selected_enum = Some(enum_name.clone());
                        state.new_enum_value.clear();
                    }
                }
            });
    });

    // New enum creation
    ui.horizontal(|ui| {
        ui.label("New:");
        ui.add(egui::TextEdit::singleline(&mut state.new_enum_name).desired_width(150.0));
        if ui.button("+ Create").clicked() && !state.new_enum_name.is_empty() {
            if !project.schema.enums.contains_key(&state.new_enum_name) {
                project.schema.enums.insert(state.new_enum_name.clone(), Vec::new());
                state.selected_enum = Some(state.new_enum_name.clone());
                state.new_enum_name.clear();
                changed = true;
            }
        }
    });

    ui.separator();

    // Selected enum details
    if let Some(enum_name) = &state.selected_enum.clone() {
        ui.horizontal(|ui| {
            ui.heading(enum_name);
            if ui.button("Delete Enum").clicked() {
                delete_enum = Some(enum_name.clone());
            }
        });

        ui.separator();

        // Enum values list
        ui.label("Values:");
        if let Some(values) = project.schema.enums.get(enum_name) {
            for (i, value) in values.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("  {} {}", i + 1, value));
                    if ui.small_button("x").clicked() {
                        to_remove = Some(i);
                    }
                });
            }
        }

        ui.separator();

        // Add new value
        ui.horizontal(|ui| {
            ui.label("Add value:");
            ui.add(egui::TextEdit::singleline(&mut state.new_enum_value).desired_width(150.0));
            if ui.button("+").clicked() && !state.new_enum_value.is_empty() {
                if let Some(values) = project.schema.enums.get(enum_name) {
                    if !values.contains(&state.new_enum_value) {
                        add_value = Some(state.new_enum_value.clone());
                    }
                }
            }
        });
    } else {
        ui.label("Select an enum or create a new one");
    }

    // Apply changes after UI
    if let Some(name) = delete_enum {
        project.schema.enums.remove(&name);
        state.selected_enum = None;
        changed = true;
    }

    if let Some(enum_name) = &state.selected_enum {
        if let Some(idx) = to_remove {
            if let Some(values) = project.schema.enums.get_mut(enum_name) {
                values.remove(idx);
                changed = true;
            }
        }

        if let Some(new_val) = add_value {
            if let Some(values) = project.schema.enums.get_mut(enum_name) {
                values.push(new_val);
                state.new_enum_value.clear();
                changed = true;
            }
        }
    }

    if changed {
        project.mark_dirty();
    }
}

#[derive(Clone, Copy)]
enum TypeCategory {
    Data,
    Embedded,
}

fn get_types_mut(project: &mut Project, category: TypeCategory) -> &mut std::collections::HashMap<String, TypeDef> {
    match category {
        TypeCategory::Data => &mut project.schema.data_types,
        TypeCategory::Embedded => &mut project.schema.embedded_types,
    }
}

fn get_types(project: &Project, category: TypeCategory) -> &std::collections::HashMap<String, TypeDef> {
    match category {
        TypeCategory::Data => &project.schema.data_types,
        TypeCategory::Embedded => &project.schema.embedded_types,
    }
}

fn render_types_tab(ui: &mut egui::Ui, state: &mut SchemaEditorState, project: &mut Project, category: TypeCategory) {
    let mut changed = false;
    let mut delete_type: Option<String> = None;

    // Type selector dropdown
    ui.horizontal(|ui| {
        ui.label("Type:");

        let current_label = state.selected_type.as_deref().unwrap_or("(select)");
        egui::ComboBox::from_id_salt("type_selector")
            .selected_text(current_label)
            .show_ui(ui, |ui| {
                let types = get_types(project, category);
                let mut type_names: Vec<_> = types.keys().cloned().collect();
                type_names.sort();
                for type_name in &type_names {
                    if ui.selectable_label(state.selected_type.as_ref() == Some(type_name), type_name).clicked() {
                        state.selected_type = Some(type_name.clone());
                        state.new_property.clear();
                        state.editing_property = None;
                    }
                }
            });
    });

    // New type creation
    ui.horizontal(|ui| {
        ui.label("New:");
        ui.add(egui::TextEdit::singleline(&mut state.new_type_name).desired_width(150.0));
        if ui.button("+ Create").clicked() && !state.new_type_name.is_empty() {
            let types = get_types_mut(project, category);
            if !types.contains_key(&state.new_type_name) {
                types.insert(state.new_type_name.clone(), TypeDef::default());
                state.selected_type = Some(state.new_type_name.clone());
                state.new_type_name.clear();
                changed = true;
            }
        }
    });

    ui.separator();

    // Selected type details
    if let Some(type_name) = &state.selected_type.clone() {
        let type_def = {
            let types = get_types(project, category);
            types.get(type_name).cloned()
        };

        if let Some(mut type_def) = type_def {
            // Header
            ui.horizontal(|ui| {
                ui.heading(type_name);
                if ui.button("Delete Type").clicked() {
                    delete_type = Some(type_name.clone());
                }
            });

            ui.separator();

            // Type settings
            ui.horizontal(|ui| {
                ui.label("Color:");
                let (r, g, b) = parse_hex_color(&type_def.color);
                let mut color = [r, g, b];
                if ui.color_edit_button_srgb(&mut color).changed() {
                    type_def.color = format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2]);
                    changed = true;
                }
            });

            // Show placeable checkbox for Data types (not embedded types)
            if matches!(category, TypeCategory::Data) {
                if ui.checkbox(&mut type_def.placeable, "Placeable in levels").changed() {
                    changed = true;
                }
            }

            ui.separator();

            // Properties section
            ui.label(egui::RichText::new("Properties").strong());

            let mut to_remove = None;
            let mut to_move_up = None;
            let mut to_move_down = None;

            for (i, prop) in type_def.properties.iter().enumerate() {
                ui.horizontal(|ui| {
                    let label = if prop.required {
                        format!("{}* ({})", prop.name, prop.prop_type.display_name())
                    } else {
                        format!("{} ({})", prop.name, prop.prop_type.display_name())
                    };
                    ui.label(&label);

                    if let Some(enum_type) = &prop.enum_type {
                        ui.label(format!("-> {}", enum_type));
                    }
                    // For arrays, show item type and optionally ref type
                    if prop.prop_type == PropType::Array {
                        if let Some(item_type) = &prop.item_type {
                            if item_type == "ref" {
                                if let Some(ref_type) = &prop.ref_type {
                                    ui.label(format!("-> [{}]", ref_type));
                                } else {
                                    ui.label("-> [ref]");
                                }
                            } else {
                                ui.label(format!("-> [{}]", item_type));
                            }
                        }
                    } else if let Some(ref_type) = &prop.ref_type {
                        ui.label(format!("-> {}", ref_type));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("x").clicked() {
                            to_remove = Some(i);
                        }
                        if i > 0 && ui.small_button("^").clicked() {
                            to_move_up = Some(i);
                        }
                        if i < type_def.properties.len() - 1 && ui.small_button("v").clicked() {
                            to_move_down = Some(i);
                        }
                        if ui.small_button("Edit").clicked() {
                            state.editing_property = Some((i, NewPropertyState::from_property_def(prop)));
                        }
                    });
                });
            }

            if let Some(idx) = to_remove {
                type_def.properties.remove(idx);
                changed = true;
            }
            if let Some(idx) = to_move_up {
                type_def.properties.swap(idx, idx - 1);
                changed = true;
            }
            if let Some(idx) = to_move_down {
                type_def.properties.swap(idx, idx + 1);
                changed = true;
            }

            ui.separator();

            // Edit existing property
            if let Some((_idx, ref mut edit_state)) = state.editing_property.clone() {
                ui.group(|ui| {
                    ui.label(egui::RichText::new(format!("Edit: {}", edit_state.name)).strong());
                    render_property_form(ui, &mut state.editing_property.as_mut().unwrap().1, project);

                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            if let Some((idx, edit_state)) = state.editing_property.take() {
                                if idx < type_def.properties.len() {
                                    type_def.properties[idx] = edit_state.to_property_def();
                                    changed = true;
                                }
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            state.editing_property = None;
                        }
                    });
                });
            } else {
                // Add new property form
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Add Property").strong());
                    render_property_form(ui, &mut state.new_property, project);

                    if ui.button("+ Add Property").clicked() && !state.new_property.name.is_empty() {
                        let exists = type_def.properties.iter().any(|p| p.name == state.new_property.name);
                        if !exists {
                            type_def.properties.push(state.new_property.to_property_def());
                            state.new_property.clear();
                            changed = true;
                        }
                    }
                });
            }

            // Save changes back
            if changed {
                let types = get_types_mut(project, category);
                types.insert(type_name.to_string(), type_def);
            }
        }
    } else {
        ui.label("Select a type or create a new one");
    }

    // Handle type deletion
    if let Some(name) = delete_type {
        let types = get_types_mut(project, category);
        types.remove(&name);
        state.selected_type = None;
        changed = true;
    }

    if changed {
        project.mark_dirty();
    }
}

fn render_property_form(ui: &mut egui::Ui, prop: &mut NewPropertyState, project: &Project) {
    egui::Grid::new("property_form")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            ui.label("Name:");
            ui.add(egui::TextEdit::singleline(&mut prop.name).desired_width(150.0));
            ui.end_row();

            ui.label("Type:");
            egui::ComboBox::from_id_salt("prop_type")
                .selected_text(prop.prop_type.display_name())
                .show_ui(ui, |ui| {
                    for pt in [
                        PropType::String,
                        PropType::Multiline,
                        PropType::Int,
                        PropType::Float,
                        PropType::Bool,
                        PropType::Enum,
                        PropType::Ref,
                        PropType::Array,
                        PropType::Embedded,
                        PropType::Point,
                        PropType::Color,
                        PropType::Sprite,
                        PropType::Dialogue,
                    ] {
                        ui.selectable_value(&mut prop.prop_type, pt, pt.display_name());
                    }
                });
            ui.end_row();

            ui.label("Required:");
            ui.checkbox(&mut prop.required, "");
            ui.end_row();

            // Type-specific fields
            match prop.prop_type {
                PropType::Enum => {
                    ui.label("Enum:");
                    egui::ComboBox::from_id_salt("enum_type")
                        .selected_text(if prop.enum_type.is_empty() { "(select)" } else { &prop.enum_type })
                        .show_ui(ui, |ui| {
                            for enum_name in project.schema.enums.keys() {
                                ui.selectable_value(&mut prop.enum_type, enum_name.clone(), enum_name);
                            }
                        });
                    ui.end_row();
                }
                PropType::Ref => {
                    ui.label("Ref Type:");
                    egui::ComboBox::from_id_salt("ref_type")
                        .selected_text(if prop.ref_type.is_empty() { "(select)" } else { &prop.ref_type })
                        .show_ui(ui, |ui| {
                            for type_name in project.schema.data_type_names() {
                                ui.selectable_value(&mut prop.ref_type, type_name.to_string(), type_name);
                            }
                        });
                    ui.end_row();
                }
                PropType::Embedded => {
                    ui.label("Embedded:");
                    egui::ComboBox::from_id_salt("embedded_type")
                        .selected_text(if prop.embedded_type.is_empty() { "(select)" } else { &prop.embedded_type })
                        .show_ui(ui, |ui| {
                            for (type_name, _) in &project.schema.embedded_types {
                                ui.selectable_value(&mut prop.embedded_type, type_name.clone(), type_name);
                            }
                        });
                    ui.end_row();
                }
                PropType::Int | PropType::Float => {
                    ui.label("Min:");
                    ui.add(egui::TextEdit::singleline(&mut prop.min).desired_width(60.0));
                    ui.end_row();

                    ui.label("Max:");
                    ui.add(egui::TextEdit::singleline(&mut prop.max).desired_width(60.0));
                    ui.end_row();
                }
                PropType::Array => {
                    ui.label("Item Type:");
                    egui::ComboBox::from_id_salt("array_item_type")
                        .selected_text(if prop.item_type.is_empty() { "string" } else { &prop.item_type })
                        .show_ui(ui, |ui| {
                            for item in ["string", "int", "float", "ref"] {
                                ui.selectable_value(&mut prop.item_type, item.to_string(), item);
                            }
                        });
                    ui.end_row();

                    // If item type is "ref", show ref type selector
                    if prop.item_type == "ref" {
                        ui.label("Ref Type:");
                        egui::ComboBox::from_id_salt("array_ref_type")
                            .selected_text(if prop.ref_type.is_empty() { "(select)" } else { &prop.ref_type })
                            .show_ui(ui, |ui| {
                                for type_name in project.schema.data_type_names() {
                                    ui.selectable_value(&mut prop.ref_type, type_name.to_string(), type_name);
                                }
                            });
                        ui.end_row();
                    }
                }
                _ => {}
            }

            ui.label("Show if:");
            ui.add(egui::TextEdit::singleline(&mut prop.show_if).desired_width(150.0));
            ui.end_row();
        });
}

fn parse_hex_color(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);
        (r, g, b)
    } else {
        (128, 128, 128)
    }
}
