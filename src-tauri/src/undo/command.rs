use crate::model::pcb::*;
use crate::model::schematic::*;
use crate::model::geometry::*;
use crate::model::library::ComponentId;
use crate::model::id::*;

pub trait Command: Send + Sync {
    fn execute(&mut self, board: &mut PcbBoard, schematic: &mut SchematicSheet) -> Result<(), String>;
    fn undo(&mut self, board: &mut PcbBoard, schematic: &mut SchematicSheet) -> Result<(), String>;
    fn name(&self) -> &str;
}

pub struct CommandHistory {
    executed: Vec<Box<dyn Command>>,
    undone: Vec<Box<dyn Command>>,
    max: usize,
}

impl CommandHistory {
    pub fn new(max: usize) -> Self {
        Self { executed: Vec::new(), undone: Vec::new(), max }
    }

    pub fn execute(&mut self, mut cmd: Box<dyn Command>, board: &mut PcbBoard, schematic: &mut SchematicSheet) -> Result<(), String> {
        cmd.execute(board, schematic)?;
        self.executed.push(cmd);
        self.undone.clear();
        if self.executed.len() > self.max {
            self.executed.remove(0);
        }
        Ok(())
    }

    pub fn undo(&mut self, board: &mut PcbBoard, schematic: &mut SchematicSheet) -> Result<(), String> {
        if let Some(mut cmd) = self.executed.pop() {
            cmd.undo(board, schematic)?;
            self.undone.push(cmd);
        }
        Ok(())
    }

    pub fn redo(&mut self, board: &mut PcbBoard, schematic: &mut SchematicSheet) -> Result<(), String> {
        if let Some(mut cmd) = self.undone.pop() {
            cmd.execute(board, schematic)?;
            self.executed.push(cmd);
        }
        Ok(())
    }

    pub fn can_undo(&self) -> bool { !self.executed.is_empty() }
    pub fn can_redo(&self) -> bool { !self.undone.is_empty() }
    pub fn clear(&mut self) { self.executed.clear(); self.undone.clear(); }
}

// ── PCB Commands ──

pub struct PlaceFootprint {
    pub component_id: ComponentId,
    pub position: Point,
    pub rotation: Rotation,
    pub result_id: Option<FootprintId>,
}

impl Command for PlaceFootprint {
    fn execute(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        let id = board.footprints.insert(FootprintInstance {
            component_id: self.component_id,
            position: self.position,
            rotation: self.rotation,
            mirrored: false,
            refdes: String::new(),
            locked: false,
        });
        self.result_id = Some(FootprintId(id));
        Ok(())
    }

    fn undo(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        if let Some(id) = self.result_id {
            board.footprints.remove(id.0);
        }
        Ok(())
    }

    fn name(&self) -> &str { "Place Footprint" }
}

pub struct RemoveFootprint {
    pub id: FootprintId,
    saved: Option<FootprintInstance>,
}

impl RemoveFootprint {
    pub fn new(id: FootprintId) -> Self { Self { id, saved: None } }
}

impl Command for RemoveFootprint {
    fn execute(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        self.saved = board.footprints.remove(self.id.0);
        Ok(())
    }

    fn undo(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        if let Some(fp) = self.saved.take() {
            board.footprints.insert_at(self.id.0, fp);
        }
        Ok(())
    }

    fn name(&self) -> &str { "Remove Footprint" }
}

pub struct RouteTraceCmd {
    pub net_id: Option<NetId>,
    pub layer: LayerId,
    pub width: f64,
    pub points: Vec<Point>,
    pub result_id: Option<TraceId>,
}

impl Command for RouteTraceCmd {
    fn execute(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        let id = board.traces.insert(Trace {
            net_id: self.net_id,
            layer: self.layer,
            width: self.width,
            points: self.points.clone(),
            attenuation_db_per_m: 0.0,
        });
        self.result_id = Some(TraceId(id));
        Ok(())
    }

    fn undo(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        if let Some(id) = self.result_id {
            board.traces.remove(id.0);
        }
        Ok(())
    }

    fn name(&self) -> &str { "Route Trace" }
}

pub struct RemoveTraceCmd {
    pub id: TraceId,
    saved: Option<Trace>,
}

impl RemoveTraceCmd {
    pub fn new(id: TraceId) -> Self { Self { id, saved: None } }
}

impl Command for RemoveTraceCmd {
    fn execute(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        self.saved = board.traces.remove(self.id.0);
        Ok(())
    }

    fn undo(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        if let Some(t) = self.saved.take() {
            board.traces.insert_at(self.id.0, t);
        }
        Ok(())
    }

    fn name(&self) -> &str { "Remove Trace" }
}

pub struct AddViaCmd {
    pub position: Point,
    pub diameter: f64,
    pub drill: f64,
    pub net_id: Option<NetId>,
    pub result_id: Option<ViaId>,
}

impl Command for AddViaCmd {
    fn execute(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        let id = board.vias.insert(Via {
            position: self.position,
            start_layer: LayerId(1),
            end_layer: LayerId(2),
            diameter: self.diameter,
            drill: self.drill,
            net_id: self.net_id,
        });
        self.result_id = Some(ViaId(id));
        Ok(())
    }

    fn undo(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        if let Some(id) = self.result_id {
            board.vias.remove(id.0);
        }
        Ok(())
    }

    fn name(&self) -> &str { "Add Via" }
}

pub struct RemoveViaCmd {
    pub id: ViaId,
    saved: Option<Via>,
}

impl RemoveViaCmd {
    pub fn new(id: ViaId) -> Self { Self { id, saved: None } }
}

impl Command for RemoveViaCmd {
    fn execute(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        self.saved = board.vias.remove(self.id.0);
        Ok(())
    }

    fn undo(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        if let Some(v) = self.saved.take() {
            board.vias.insert_at(self.id.0, v);
        }
        Ok(())
    }

    fn name(&self) -> &str { "Remove Via" }
}

pub struct DrawZoneCmd {
    pub layer: LayerId,
    pub outline: Vec<Point>,
    pub result_id: Option<ZoneId>,
}

impl Command for DrawZoneCmd {
    fn execute(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        let id = board.zones.insert(Zone {
            net_id: None,
            layer: self.layer,
            outline: self.outline.clone(),
            clearance: board.design_rules.min_clearance,
            thermal_relief: true,
        });
        self.result_id = Some(ZoneId(id));
        Ok(())
    }

    fn undo(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        if let Some(id) = self.result_id {
            board.zones.remove(id.0);
        }
        Ok(())
    }

    fn name(&self) -> &str { "Draw Zone" }
}

pub struct MoveElementCmd {
    pub element_type: String,
    pub element_id: usize,
    pub delta: Point,
}

impl Command for MoveElementCmd {
    fn execute(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        match self.element_type.as_str() {
            "footprint" => {
                if let Some(fp) = board.footprints.get_mut(self.element_id) {
                    fp.position.x += self.delta.x;
                    fp.position.y += self.delta.y;
                }
            }
            "trace" => {
                if let Some(t) = board.traces.get_mut(self.element_id) {
                    for p in &mut t.points {
                        p.x += self.delta.x;
                        p.y += self.delta.y;
                    }
                }
            }
            "via" => {
                if let Some(v) = board.vias.get_mut(self.element_id) {
                    v.position.x += self.delta.x;
                    v.position.y += self.delta.y;
                }
            }
            _ => return Err(format!("Unknown element type: {}", self.element_type)),
        }
        Ok(())
    }

    fn undo(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        let neg = Point { x: -self.delta.x, y: -self.delta.y };
        let mut rev = MoveElementCmd {
            element_type: self.element_type.clone(),
            element_id: self.element_id,
            delta: neg,
        };
        rev.execute(board, _schematic)
    }

    fn name(&self) -> &str { "Move Element" }
}

pub struct SetBoardOutlineCmd {
    pub shape: Vec<Vec<Point>>,
    pub saved: Option<Option<BoardOutline>>,
}

impl SetBoardOutlineCmd {
    pub fn new(shape: Vec<Vec<Point>>) -> Self { Self { shape, saved: None } }
}

impl Command for SetBoardOutlineCmd {
    fn execute(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        self.saved = Some(board.outline.take());
        board.outline = Some(BoardOutline { shape: self.shape.clone(), cutouts: vec![] });
        Ok(())
    }

    fn undo(&mut self, board: &mut PcbBoard, _schematic: &mut SchematicSheet) -> Result<(), String> {
        board.outline = self.saved.take().unwrap_or(None);
        Ok(())
    }

    fn name(&self) -> &str { "Set Board Outline" }
}

// ── Schematic Commands ──

pub struct PlaceSymbolCmd {
    pub component_id: ComponentId,
    pub position: Point,
    pub refdes: String,
    pub result_id: Option<SymbolId>,
}

impl Command for PlaceSymbolCmd {
    fn execute(&mut self, _board: &mut PcbBoard, schematic: &mut SchematicSheet) -> Result<(), String> {
        let id = schematic.symbols.insert(SymbolInstance {
            component_id: self.component_id,
            position: self.position,
            rotation: Rotation(0.0),
            mirrored: false,
            refdes: self.refdes.clone(),
            value: String::new(),
        });
        self.result_id = Some(SymbolId(id));
        Ok(())
    }

    fn undo(&mut self, _board: &mut PcbBoard, schematic: &mut SchematicSheet) -> Result<(), String> {
        if let Some(id) = self.result_id {
            schematic.symbols.remove(id.0);
        }
        Ok(())
    }

    fn name(&self) -> &str { "Place Symbol" }
}

pub struct RemoveSymbolCmd {
    pub id: SymbolId,
    saved: Option<SymbolInstance>,
}

impl RemoveSymbolCmd {
    pub fn new(id: SymbolId) -> Self { Self { id, saved: None } }
}

impl Command for RemoveSymbolCmd {
    fn execute(&mut self, _board: &mut PcbBoard, schematic: &mut SchematicSheet) -> Result<(), String> {
        self.saved = schematic.symbols.remove(self.id.0);
        Ok(())
    }

    fn undo(&mut self, _board: &mut PcbBoard, schematic: &mut SchematicSheet) -> Result<(), String> {
        if let Some(s) = self.saved.take() {
            schematic.symbols.insert_at(self.id.0, s);
        }
        Ok(())
    }

    fn name(&self) -> &str { "Remove Symbol" }
}

pub struct DrawWireCmd {
    pub points: Vec<Point>,
    pub width: f64,
    pub result_id: Option<WireId>,
}

impl Command for DrawWireCmd {
    fn execute(&mut self, _board: &mut PcbBoard, schematic: &mut SchematicSheet) -> Result<(), String> {
        let id = schematic.wires.insert(Wire {
            points: self.points.clone(),
            net_id: None,
            width: self.width,
        });
        self.result_id = Some(WireId(id));
        Ok(())
    }

    fn undo(&mut self, _board: &mut PcbBoard, schematic: &mut SchematicSheet) -> Result<(), String> {
        if let Some(id) = self.result_id {
            schematic.wires.remove(id.0);
        }
        Ok(())
    }

    fn name(&self) -> &str { "Draw Wire" }
}

pub struct AddNetLabelCmd {
    pub name: String,
    pub position: Point,
    pub result_id: Option<usize>,
}

impl Command for AddNetLabelCmd {
    fn execute(&mut self, _board: &mut PcbBoard, schematic: &mut SchematicSheet) -> Result<(), String> {
        let id = schematic.net_labels.insert(NetLabel {
            name: self.name.clone(),
            position: self.position,
        });
        self.result_id = Some(id);
        Ok(())
    }

    fn undo(&mut self, _board: &mut PcbBoard, schematic: &mut SchematicSheet) -> Result<(), String> {
        if let Some(id) = self.result_id {
            schematic.net_labels.remove(id);
        }
        Ok(())
    }

    fn name(&self) -> &str { "Add Net Label" }
}

pub struct MoveSchematicElementCmd {
    pub element_type: String,
    pub element_id: usize,
    pub delta: Point,
}

impl Command for MoveSchematicElementCmd {
    fn execute(&mut self, _board: &mut PcbBoard, schematic: &mut SchematicSheet) -> Result<(), String> {
        match self.element_type.as_str() {
            "symbol" => {
                if let Some(s) = schematic.symbols.get_mut(self.element_id) {
                    s.position.x += self.delta.x;
                    s.position.y += self.delta.y;
                }
            }
            "wire" => {
                if let Some(w) = schematic.wires.get_mut(self.element_id) {
                    for p in &mut w.points {
                        p.x += self.delta.x;
                        p.y += self.delta.y;
                    }
                }
            }
            "net_label" => {
                if let Some(nl) = schematic.net_labels.get_mut(self.element_id) {
                    nl.position.x += self.delta.x;
                    nl.position.y += self.delta.y;
                }
            }
            _ => return Err(format!("Unknown element type: {}", self.element_type)),
        }
        Ok(())
    }

    fn undo(&mut self, _board: &mut PcbBoard, schematic: &mut SchematicSheet) -> Result<(), String> {
        let neg = Point { x: -self.delta.x, y: -self.delta.y };
        let mut rev = MoveSchematicElementCmd {
            element_type: self.element_type.clone(),
            element_id: self.element_id,
            delta: neg,
        };
        rev.execute(_board, schematic)
    }

    fn name(&self) -> &str { "Move Schematic Element" }
}
