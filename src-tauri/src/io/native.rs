use std::collections::HashMap;
use crate::model::id::SheetId;
use crate::model::pcb::PcbBoard;
use crate::model::schematic::SchematicSheet;
use crate::model::project::Project;

#[derive(serde::Serialize, serde::Deserialize)]
struct ProjectData {
    project: Project,
    board: PcbBoard,
    schematics: HashMap<SheetId, SchematicSheet>,
}

pub fn save(project: &Project, board: &PcbBoard, schematics: &HashMap<SheetId, SchematicSheet>, path: &str) -> Result<(), String> {
    let data = ProjectData {
        project: project.clone(),
        board: board.clone(),
        schematics: schematics.clone(),
    };
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load(path: &str) -> Result<(Project, PcbBoard, HashMap<SheetId, SchematicSheet>), String> {
    let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let data: ProjectData = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok((data.project, data.board, data.schematics))
}
