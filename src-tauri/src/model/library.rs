use serde::{Deserialize, Serialize};
use crate::model::geometry::*;
use crate::model::id::LayerId;
pub use crate::model::id::ComponentId;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: ComponentId,
    pub name: String,
    pub description: String,
    pub manufacturer: Option<String>,
    pub category: Option<String>,
    pub symbol: SymbolDef,
    pub footprint: FootprintDef,
    pub model_3d: Option<Model3D>,
    pub parameters: HashMap<String, String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolDef {
    pub pins: Vec<PinDef>,
    pub body: Vec<SymbolGraphics>,
    pub origin: Point,
}

impl Default for SymbolDef {
    fn default() -> Self {
        Self { pins: Vec::new(), body: Vec::new(), origin: Point::new(0.0, 0.0) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinDef {
    pub name: String,
    pub number: String,
    pub position: Point,
    pub length: f64,
    pub electrical_type: PinType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinType {
    Input, Output, Bidirectional, Power, Gnd, Passive, OpenCollector, OpenEmitter, NotConnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolGraphics {
    Rectangle { position: Point, size: Size, fill: String, stroke: String },
    Circle { center: Point, radius: f64, fill: String, stroke: String },
    Arc { center: Point, radius: f64, start_angle: f64, end_angle: f64, stroke: String },
    Line { start: Point, end: Point, stroke: String, width: f64 },
    Text { text: String, position: Point, size: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootprintDef {
    pub pads: Vec<PadDef>,
    pub outline: Vec<Point>,
    pub courtyard: Vec<Point>,
}

impl Default for FootprintDef {
    fn default() -> Self {
        Self { pads: Vec::new(), outline: Vec::new(), courtyard: Vec::new() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PadDef {
    pub number: String,
    pub shape: PadShape,
    pub position: Point,
    pub size: Size,
    pub layer: LayerId,
    pub drill: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PadShape {
    Round, Rect, Oval, RoundRect, Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model3D {
    pub path: String,
    pub offset: Point,
    pub rotation: Rotation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSummary {
    pub id: ComponentId,
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub manufacturer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub category: Option<String>,
    pub page: usize,
    pub per_page: usize,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self { text: String::new(), category: None, page: 0, per_page: 20 }
    }
}

#[async_trait::async_trait]
pub trait ComponentRepository: Send + Sync {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<ComponentSummary>, String>;
    async fn get_by_id(&self, id: ComponentId) -> Result<Option<Component>, String>;
    async fn save(&self, component: &Component) -> Result<(), String>;
    async fn delete(&self, id: ComponentId) -> Result<(), String>;
    async fn get_categories(&self) -> Result<Vec<Category>, String>;
}
