use serde::{Deserialize, Serialize};
use crate::model::arena::Arena;
use crate::model::geometry::*;
use crate::model::id::*;
use crate::model::library::ComponentId;
use crate::model::netlist::Netlist;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcbBoard {
    pub footprints: Arena<FootprintInstance>,
    pub traces: Arena<Trace>,
    pub vias: Arena<Via>,
    pub zones: Arena<Zone>,
    #[serde(default)]
    pub diff_pairs: Vec<DifferentialPair>,
    #[serde(default)]
    pub flex_zones: Vec<FlexZone>,
    pub outline: Option<BoardOutline>,
    pub layers: LayerStack,
    pub design_rules: DesignRules,
    pub netlist: Netlist,
}

impl Default for PcbBoard {
    fn default() -> Self {
        Self {
            footprints: Arena::new(),
            traces: Arena::new(),
            vias: Arena::new(),
            zones: Arena::new(),
            diff_pairs: vec![],
            flex_zones: vec![],
            outline: None,
            layers: LayerStack::default_2layer(),
            design_rules: DesignRules::default(),
            netlist: Netlist { nets: vec![] },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferentialPair {
    pub name: String,
    pub p_net: NetId,
    pub n_net: NetId,
    pub width: f64,
    pub gap: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootprintInstance {
    pub component_id: ComponentId,
    pub position: Point,
    pub rotation: Rotation,
    pub mirrored: bool,
    pub refdes: String,
    pub locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub net_id: Option<NetId>,
    pub layer: LayerId,
    pub width: f64,
    pub points: Vec<Point>,
    #[serde(default)]
    pub attenuation_db_per_m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Via {
    pub position: Point,
    pub start_layer: LayerId,
    pub end_layer: LayerId,
    pub diameter: f64,
    pub drill: f64,
    pub net_id: Option<NetId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub net_id: Option<NetId>,
    pub layer: LayerId,
    pub outline: Vec<Point>,
    pub clearance: f64,
    pub thermal_relief: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardOutline {
    pub shape: Vec<Vec<Point>>,
    pub cutouts: Vec<Vec<Point>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlexZone {
    pub name: String,
    pub region: Vec<Point>,
    pub bend_radius_mm: f64,
    pub num_bends: u32,
    #[serde(default)]
    pub layer_keepout: Vec<LayerId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStack {
    pub layers: Vec<LayerDef>,
    #[serde(default = "default_dielectrics")]
    pub dielectrics: Vec<Dielectric>,
    #[serde(default = "default_copper_weight")]
    pub copper_weight_oz: f64,
}

fn default_dielectrics() -> Vec<Dielectric> {
    vec![Dielectric { name: "FR-4".into(), er: 4.5, thickness_mm: 1.6, loss_tangent: 0.02 }]
}

fn default_copper_weight() -> f64 { 1.0 }

impl LayerStack {
    pub fn default_2layer() -> Self {
        Self {
            layers: vec![
                LayerDef { id: LayerId(1), name: "Top".into(), kind: LayerKind::Top, color: "#D47A2F".into() },
                LayerDef { id: LayerId(2), name: "Bottom".into(), kind: LayerKind::Bottom, color: "#A0793F".into() },
                LayerDef { id: LayerId(10), name: "Top Solder Mask".into(), kind: LayerKind::SolderMask, color: "#30853A".into() },
                LayerDef { id: LayerId(11), name: "Bottom Solder Mask".into(), kind: LayerKind::SolderMask, color: "#25682D".into() },
                LayerDef { id: LayerId(20), name: "Top Silkscreen".into(), kind: LayerKind::Silkscreen, color: "#F0F0F0".into() },
                LayerDef { id: LayerId(21), name: "Bottom Silkscreen".into(), kind: LayerKind::Silkscreen, color: "#C0C0C0".into() },
            ],
            dielectrics: vec![
                Dielectric { name: "FR-4".into(), er: 4.5, thickness_mm: 1.6, loss_tangent: 0.02 },
            ],
            copper_weight_oz: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dielectric {
    pub name: String,
    pub er: f64,
    pub thickness_mm: f64,
    pub loss_tangent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpedanceResult {
    pub trace_width_mm: f64,
    pub impedance_ohms: f64,
    pub trace_thickness_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerDef {
    pub id: LayerId,
    pub name: String,
    pub kind: LayerKind,
    pub color: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerKind {
    Top,
    Bottom,
    Inner,
    SolderMask,
    Silkscreen,
    Paste,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignRules {
    pub min_trace_width: f64,
    pub min_clearance: f64,
    pub min_annular_ring: f64,
    pub min_hole_size: f64,
    pub default_trace_width: f64,
    pub default_via_diameter: f64,
    pub default_via_drill: f64,
}

impl Default for DesignRules {
    fn default() -> Self {
        Self {
            min_trace_width: 0.15,
            min_clearance: 0.15,
            min_annular_ring: 0.15,
            min_hole_size: 0.3,
            default_trace_width: 0.25,
            default_via_diameter: 0.6,
            default_via_drill: 0.3,
        }
    }
}
