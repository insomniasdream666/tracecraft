use serde::{Deserialize, Serialize};
use crate::model::arena::Arena;
use crate::model::geometry::*;
use crate::model::id::*;
use crate::model::library::ComponentId;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchematicSheet {
    pub id: SheetId,
    pub name: String,
    pub symbols: Arena<SymbolInstance>,
    pub wires: Arena<Wire>,
    pub junctions: Arena<Junction>,
    pub net_labels: Arena<NetLabel>,
    pub buses: Arena<Bus>,
    pub text_annotations: Arena<TextAnnotation>,
    pub hier_sheets: Arena<HierarchicalSheet>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInstance {
    pub component_id: ComponentId,
    pub position: Point,
    pub rotation: Rotation,
    pub mirrored: bool,
    pub refdes: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wire {
    pub points: Vec<Point>,
    pub net_id: Option<NetId>,
    pub width: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetLabel {
    pub name: String,
    pub position: Point,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Junction {
    pub position: Point,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bus {
    pub name: String,
    pub members: Vec<String>,
    pub wires: Vec<Vec<Point>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextAnnotation {
    pub text: String,
    pub position: Point,
    pub size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalSheet {
    pub id: SheetId,
    pub name: String,
    pub child_sheet_id: SheetId,
    pub pins: Vec<HierarchicalPin>,
    pub position: Point,
    pub size: Size,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalPin {
    pub name: String,
    pub pin_number: String,
    pub position: Point,
}
