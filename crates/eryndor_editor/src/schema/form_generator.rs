use super::{DialogueTree, PropType, PropertyDef, Schema, SpriteData, TypeDef, Value};
use bevy_egui::egui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Context needed for generating forms
pub struct FormContext<'a> {
    pub schema: &'a Schema,
    pub all_instances: &'a HashMap<String, Vec<DataInstance>>,
}

/// A data instance with properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataInstance {
    pub id: Uuid,
    pub type_name: String,
    pub properties: HashMap<String, Value>,
}

impl DataInstance {
    pub fn new(type_name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            type_name,
            properties: HashMap::new(),
        }
    }

    pub fn get_display_name(&self) -> String {
        self.properties
            .get("name")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{} ({})", self.type_name, &self.id.to_string()[..8]))
    }

    pub fn get_string(&self, key: &str) -> String {
        self.properties
            .get(key)
            .and_then(|v| v.as_string())
            .unwrap_or("")
            .to_string()
    }

    pub fn set_string(&mut self, key: &str, value: String) {
        self.properties.insert(key.to_string(), Value::String(value));
    }

    pub fn get_int(&self, key: &str) -> i64 {
        self.properties
            .get(key)
            .and_then(|v| v.as_int())
            .unwrap_or(0)
    }

    pub fn set_int(&mut self, key: &str, value: i64) {
        self.properties.insert(key.to_string(), Value::Int(value));
    }

    pub fn get_float(&self, key: &str) -> f64 {
        self.properties
            .get(key)
            .and_then(|v| v.as_float())
            .unwrap_or(0.0)
    }

    pub fn set_float(&mut self, key: &str, value: f64) {
        self.properties.insert(key.to_string(), Value::Float(value));
    }

    pub fn get_bool(&self, key: &str) -> bool {
        self.properties
            .get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.properties.insert(key.to_string(), Value::Bool(value));
    }
}

/// Generate a form for editing an instance based on schema
pub fn render_instance_form(
    ui: &mut egui::Ui,
    schema: &Schema,
    type_def: &TypeDef,
    instance: &mut DataInstance,
    ctx: &FormContext,
) -> FormResult {
    let mut result = FormResult::default();

    for prop in &type_def.properties {
        // Check conditional visibility
        if let Some(condition) = &prop.show_if {
            if !evaluate_condition(condition, &instance.properties) {
                continue;
            }
        }

        ui.horizontal(|ui| {
            let label = if prop.required {
                format!("{}*", prop.name)
            } else {
                prop.name.clone()
            };
            ui.label(&label);

            match prop.prop_type {
                PropType::String => {
                    let mut value = instance.get_string(&prop.name);
                    if ui.text_edit_singleline(&mut value).changed() {
                        instance.set_string(&prop.name, value);
                        result.changed = true;
                    }
                }
                PropType::Multiline => {
                    let mut value = instance.get_string(&prop.name);
                    if ui
                        .add(egui::TextEdit::multiline(&mut value).desired_rows(4))
                        .changed()
                    {
                        instance.set_string(&prop.name, value);
                        result.changed = true;
                    }
                }
                PropType::Int => {
                    let mut value = instance.get_int(&prop.name);
                    let mut drag = egui::DragValue::new(&mut value);
                    if let Some(min) = prop.min {
                        drag = drag.range(min as i64..=i64::MAX);
                    }
                    if let Some(max) = prop.max {
                        drag = drag.range(i64::MIN..=max as i64);
                    }
                    if ui.add(drag).changed() {
                        instance.set_int(&prop.name, value);
                        result.changed = true;
                    }
                }
                PropType::Float => {
                    let mut value = instance.get_float(&prop.name);
                    let mut drag = egui::DragValue::new(&mut value).speed(0.1);
                    if let Some(min) = prop.min {
                        drag = drag.range(min..=f64::MAX);
                    }
                    if let Some(max) = prop.max {
                        drag = drag.range(f64::MIN..=max);
                    }
                    if ui.add(drag).changed() {
                        instance.set_float(&prop.name, value);
                        result.changed = true;
                    }
                }
                PropType::Bool => {
                    let mut value = instance.get_bool(&prop.name);
                    if ui.checkbox(&mut value, "").changed() {
                        instance.set_bool(&prop.name, value);
                        result.changed = true;
                    }
                }
                PropType::Enum => {
                    if let Some(enum_name) = &prop.enum_type {
                        if let Some(enum_values) = schema.get_enum(enum_name) {
                            let current = instance.get_string(&prop.name);
                            let mut selected = current.clone();

                            egui::ComboBox::from_id_salt(&prop.name)
                                .selected_text(&selected)
                                .show_ui(ui, |ui| {
                                    for value in enum_values {
                                        if ui
                                            .selectable_value(&mut selected, value.clone(), value)
                                            .clicked()
                                        {
                                            result.changed = true;
                                        }
                                    }
                                });

                            if selected != current {
                                instance.set_string(&prop.name, selected);
                            }
                        }
                    }
                }
                PropType::Ref => {
                    result.changed |= render_ref_dropdown(ui, prop, instance, ctx);
                }
                PropType::Array => {
                    result.changed |= render_array_editor(ui, schema, prop, instance, ctx);
                }
                PropType::Embedded => {
                    // Embedded objects: show nested form if embedded_type is defined
                    if let Some(embedded_type_name) = &prop.embedded_type {
                        if let Some(embedded_def) = schema.embedded_types.get(embedded_type_name) {
                            ui.collapsing(format!("{} (embedded)", prop.name), |ui| {
                                // Get or create embedded object
                                let obj = instance
                                    .properties
                                    .entry(prop.name.clone())
                                    .or_insert_with(|| Value::Object(std::collections::HashMap::new()));

                                if let Value::Object(map) = obj {
                                    for embedded_prop in &embedded_def.properties {
                                        ui.horizontal(|ui| {
                                            ui.label(&embedded_prop.name);
                                            match embedded_prop.prop_type {
                                                PropType::String | PropType::Multiline => {
                                                    let mut val = map.get(&embedded_prop.name)
                                                        .and_then(|v| v.as_string())
                                                        .unwrap_or("")
                                                        .to_string();
                                                    if ui.text_edit_singleline(&mut val).changed() {
                                                        map.insert(embedded_prop.name.clone(), Value::String(val));
                                                        result.changed = true;
                                                    }
                                                }
                                                PropType::Int => {
                                                    let mut val = map.get(&embedded_prop.name)
                                                        .and_then(|v| v.as_int())
                                                        .unwrap_or(0);
                                                    if ui.add(egui::DragValue::new(&mut val)).changed() {
                                                        map.insert(embedded_prop.name.clone(), Value::Int(val));
                                                        result.changed = true;
                                                    }
                                                }
                                                PropType::Float => {
                                                    let mut val = map.get(&embedded_prop.name)
                                                        .and_then(|v| v.as_float())
                                                        .unwrap_or(0.0);
                                                    if ui.add(egui::DragValue::new(&mut val).speed(0.1)).changed() {
                                                        map.insert(embedded_prop.name.clone(), Value::Float(val));
                                                        result.changed = true;
                                                    }
                                                }
                                                PropType::Bool => {
                                                    let mut val = map.get(&embedded_prop.name)
                                                        .and_then(|v| v.as_bool())
                                                        .unwrap_or(false);
                                                    if ui.checkbox(&mut val, "").changed() {
                                                        map.insert(embedded_prop.name.clone(), Value::Bool(val));
                                                        result.changed = true;
                                                    }
                                                }
                                                _ => {
                                                    ui.label(format!("({:?})", embedded_prop.prop_type));
                                                }
                                            }
                                        });
                                    }
                                }
                            });
                        } else {
                            ui.label(format!("(unknown embedded type: {})", embedded_type_name));
                        }
                    } else {
                        ui.label("(embedded - no type specified)");
                    }
                }
                PropType::Point => {
                    result.changed |= render_point_editor(ui, prop, instance);
                }
                PropType::Color => {
                    result.changed |= render_color_editor(ui, prop, instance);
                }
                PropType::Sprite => {
                    let sprite_result = render_sprite_field(ui, prop, instance);
                    result.changed |= sprite_result.0;
                    if sprite_result.1 {
                        result.open_sprite_editor = Some((prop.name.clone(), instance.id));
                    }
                }
                PropType::Dialogue => {
                    let dialogue_result = render_dialogue_field(ui, prop, instance);
                    result.changed |= dialogue_result.0;
                    if dialogue_result.1 {
                        result.open_dialogue_editor = Some((prop.name.clone(), instance.id));
                    }
                }
            }
        });
    }

    result
}

/// Render a dropdown for reference properties
fn render_ref_dropdown(
    ui: &mut egui::Ui,
    prop: &PropertyDef,
    instance: &mut DataInstance,
    ctx: &FormContext,
) -> bool {
    let mut changed = false;

    if let Some(ref_type) = &prop.ref_type {
        let current_id = instance
            .properties
            .get(&prop.name)
            .and_then(|v| v.as_string())
            .unwrap_or("")
            .to_string();

        let instances = ctx.all_instances.get(ref_type);
        let current_name = instances
            .and_then(|list| list.iter().find(|i| i.id.to_string() == current_id))
            .map(|i| i.get_display_name())
            .unwrap_or_else(|| "(none)".to_string());

        egui::ComboBox::from_id_salt(&prop.name)
            .selected_text(&current_name)
            .show_ui(ui, |ui| {
                // None option
                if ui.selectable_label(current_id.is_empty(), "(none)").clicked() {
                    instance
                        .properties
                        .insert(prop.name.clone(), Value::String(String::new()));
                    changed = true;
                }

                // List all instances of the referenced type
                if let Some(instances) = instances {
                    for ref_instance in instances {
                        let id_str = ref_instance.id.to_string();
                        if ui
                            .selectable_label(
                                current_id == id_str,
                                ref_instance.get_display_name(),
                            )
                            .clicked()
                        {
                            instance
                                .properties
                                .insert(prop.name.clone(), Value::String(id_str));
                            changed = true;
                        }
                    }
                }
            });
    }

    changed
}

/// Render an array editor
fn render_array_editor(
    ui: &mut egui::Ui,
    _schema: &Schema,
    prop: &PropertyDef,
    instance: &mut DataInstance,
    ctx: &FormContext,
) -> bool {
    let mut changed = false;
    let is_ref_array = prop.item_type.as_deref() == Some("ref");
    let ref_type = prop.ref_type.as_deref();

    ui.vertical(|ui| {
        let array = instance
            .properties
            .entry(prop.name.clone())
            .or_insert_with(|| Value::Array(Vec::new()));

        if let Value::Array(items) = array {
            let mut to_remove = None;

            for (i, item) in items.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("[{}]", i));

                    if is_ref_array {
                        // Show display name for references
                        if let Value::String(ref_id) = item {
                            let display = if ref_id.is_empty() {
                                "(none)".to_string()
                            } else if let Some(ref_type_name) = ref_type {
                                ctx.all_instances
                                    .get(ref_type_name)
                                    .and_then(|list| list.iter().find(|i| i.id.to_string() == *ref_id))
                                    .map(|i| i.get_display_name())
                                    .unwrap_or_else(|| format!("(unknown: {})", &ref_id[..8.min(ref_id.len())]))
                            } else {
                                ref_id.clone()
                            };
                            ui.label(display);
                        }
                    } else {
                        match item {
                            Value::String(s) => {
                                ui.label(s);
                            }
                            _ => {
                                ui.label(format!("{:?}", item));
                            }
                        }
                    }

                    if ui.small_button("x").clicked() {
                        to_remove = Some(i);
                    }
                });
            }

            if let Some(idx) = to_remove {
                items.remove(idx);
                changed = true;
            }

            // Add button with reference picker for ref arrays
            if is_ref_array {
                if let Some(ref_type_name) = ref_type {
                    let instances = ctx.all_instances.get(ref_type_name);
                    egui::ComboBox::from_id_salt(format!("{}_{}_add", prop.name, items.len()))
                        .selected_text("+ Add reference...")
                        .show_ui(ui, |ui| {
                            if let Some(instances) = instances {
                                for ref_instance in instances {
                                    if ui.selectable_label(false, ref_instance.get_display_name()).clicked() {
                                        items.push(Value::String(ref_instance.id.to_string()));
                                        changed = true;
                                    }
                                }
                            }
                        });
                } else {
                    ui.label("(no ref type specified)");
                }
            } else {
                if ui.small_button("+ Add").clicked() {
                    items.push(Value::String(String::new()));
                    changed = true;
                }
            }
        }
    });

    changed
}

/// Render a point (x, y) editor
fn render_point_editor(
    ui: &mut egui::Ui,
    prop: &PropertyDef,
    instance: &mut DataInstance,
) -> bool {
    let mut changed = false;

    // Get current point value as an object with x and y
    let point = instance
        .properties
        .entry(prop.name.clone())
        .or_insert_with(|| {
            let mut map = HashMap::new();
            map.insert("x".to_string(), Value::Float(0.0));
            map.insert("y".to_string(), Value::Float(0.0));
            Value::Object(map)
        });

    if let Value::Object(map) = point {
        let mut x = map.get("x").and_then(|v| v.as_float()).unwrap_or(0.0);
        let mut y = map.get("y").and_then(|v| v.as_float()).unwrap_or(0.0);

        ui.horizontal(|ui| {
            if ui.add(egui::DragValue::new(&mut x).prefix("x: ").speed(1.0)).changed() {
                map.insert("x".to_string(), Value::Float(x));
                changed = true;
            }
            if ui.add(egui::DragValue::new(&mut y).prefix("y: ").speed(1.0)).changed() {
                map.insert("y".to_string(), Value::Float(y));
                changed = true;
            }
        });
    }

    changed
}

/// Render a color editor with color picker
fn render_color_editor(
    ui: &mut egui::Ui,
    prop: &PropertyDef,
    instance: &mut DataInstance,
) -> bool {
    let mut changed = false;

    // Get current color value as hex string
    let current_hex = instance
        .properties
        .get(&prop.name)
        .and_then(|v| v.as_string())
        .unwrap_or("#808080")
        .to_string();

    // Parse hex to RGB u8 values
    let (r, g, b) = parse_hex_color(&current_hex);
    let mut color = [r, g, b];

    if ui.color_edit_button_srgb(&mut color).changed() {
        let new_hex = format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2]);
        instance.properties.insert(prop.name.clone(), Value::String(new_hex));
        changed = true;
    }

    // Also show hex input
    let mut hex_str = current_hex;
    if ui.add(egui::TextEdit::singleline(&mut hex_str).desired_width(80.0)).changed() {
        instance.properties.insert(prop.name.clone(), Value::String(hex_str));
        changed = true;
    }

    changed
}

/// Parse a hex color string like "#ff0000" into RGB u8 values
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

/// Result from form rendering
#[derive(Default)]
pub struct FormResult {
    /// Whether any field changed
    pub changed: bool,
    /// Sprite editor to open: (property_name, instance_id)
    pub open_sprite_editor: Option<(String, Uuid)>,
    /// Dialogue editor to open: (property_name, instance_id)
    pub open_dialogue_editor: Option<(String, Uuid)>,
}

/// Render a sprite property field with edit button
/// Returns (changed, open_editor)
fn render_sprite_field(
    ui: &mut egui::Ui,
    prop: &PropertyDef,
    instance: &mut DataInstance,
) -> (bool, bool) {
    let mut changed = false;
    let mut open_editor = false;

    // Get current sprite data
    let sprite_data = instance
        .properties
        .get(&prop.name)
        .and_then(|v| SpriteData::from_value(v));

    ui.vertical(|ui| {
        if let Some(data) = &sprite_data {
            // Show summary
            if !data.sheet_path.is_empty() {
                ui.label(format!("Sheet: {}", data.sheet_path));
                ui.label(format!("Frame: {}x{}", data.frame_width, data.frame_height));
                ui.label(format!("Animations: {}", data.animations.len()));
            } else {
                ui.label("(no sprite set)");
            }
        } else {
            ui.label("(no sprite set)");
        }

        // Edit button
        if ui.button("Edit Sprite...").clicked() {
            // Initialize sprite data if not present
            if sprite_data.is_none() {
                let new_data = SpriteData::default();
                instance.properties.insert(prop.name.clone(), new_data.to_value());
                changed = true;
            }
            open_editor = true;
        }
    });

    (changed, open_editor)
}

/// Render a dialogue property field with edit button
/// Returns (changed, open_editor)
fn render_dialogue_field(
    ui: &mut egui::Ui,
    prop: &PropertyDef,
    instance: &mut DataInstance,
) -> (bool, bool) {
    let mut changed = false;
    let mut open_editor = false;

    // Get current dialogue data
    let dialogue_data = instance
        .properties
        .get(&prop.name)
        .and_then(|v| DialogueTree::from_value(v));

    ui.vertical(|ui| {
        if let Some(data) = &dialogue_data {
            // Show summary
            if !data.name.is_empty() {
                ui.label(format!("Dialogue: {}", data.name));
            } else {
                ui.label("Dialogue: (unnamed)");
            }
            ui.label(format!("Nodes: {}", data.nodes.len()));
        } else {
            ui.label("(no dialogue set)");
        }

        // Edit button
        if ui.button("Edit Dialogue...").clicked() {
            // Initialize dialogue data if not present
            if dialogue_data.is_none() {
                let new_data = DialogueTree::new();
                instance.properties.insert(prop.name.clone(), new_data.to_value());
                changed = true;
            }
            open_editor = true;
        }
    });

    (changed, open_editor)
}

/// Evaluate a condition expression (simple implementation)
fn evaluate_condition(condition: &str, properties: &HashMap<String, Value>) -> bool {
    // Simple parser for "property == value" conditions
    if let Some((prop_name, expected)) = condition.split_once("==") {
        let prop_name = prop_name.trim();
        let expected = expected.trim();

        if let Some(value) = properties.get(prop_name) {
            match value {
                Value::String(s) => return s == expected,
                Value::Bool(b) => return b.to_string() == expected,
                Value::Int(i) => return i.to_string() == expected,
                Value::Float(f) => return f.to_string() == expected,
                _ => return false,
            }
        }
    }

    // If we can't parse the condition, default to showing the property
    true
}
