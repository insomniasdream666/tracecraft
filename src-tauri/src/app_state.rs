use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use parking_lot::RwLock;
use crate::model::project::*;
use crate::model::schematic::SchematicSheet;
use crate::model::pcb::*;
use crate::model::id::SheetId;
use crate::model::library::*;
use crate::undo::command::CommandHistory;
use rusqlite::Connection;
use tauri::AppHandle;
use std::collections::HashMap;

pub struct AppState {
    pub project: RwLock<Option<Project>>,
    pub board: RwLock<PcbBoard>,
    pub schematics: RwLock<HashMap<SheetId, SchematicSheet>>,
    pub command_history: RwLock<CommandHistory>,
    pub library: Arc<dyn ComponentRepository>,
    pub settings: RwLock<Settings>,
    pub ai_engine: RwLock<crate::ai::AIEngine>,
    pub db: std::sync::Mutex<Connection>,
    pub app_handle: RwLock<Option<AppHandle>>,
    pub routing_cancelled: AtomicBool,
}

use crate::ai::AIEngine;

impl AppState {
    pub fn new(db: Connection, library: Arc<dyn ComponentRepository>) -> Self {
        Self {
            project: RwLock::new(None),
            board: RwLock::new(PcbBoard::default()),
            schematics: RwLock::new(HashMap::new()),
            command_history: RwLock::new(CommandHistory::new(1000)),
            library,
            settings: RwLock::new(Settings::default()),
            ai_engine: RwLock::new(AIEngine::new()),
            db: std::sync::Mutex::new(db),
            app_handle: RwLock::new(None),
            routing_cancelled: AtomicBool::new(false),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub grid_size: f64,
    pub snap_to_grid: bool,
    pub dark_mode: bool,
    pub recent_files: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            grid_size: 2.54,
            snap_to_grid: true,
            dark_mode: true,
            recent_files: Vec::new(),
        }
    }
}
