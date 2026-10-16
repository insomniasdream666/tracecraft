use tauri::State;
use crate::app_state::AppState;
use crate::model::library::*;
use crate::model::pcb::{LayerStack, Dielectric, LayerDef, LayerKind};
use crate::model::id::LayerId;
use std::collections::HashMap;

#[tauri::command]
pub async fn search_components(
    state: State<'_, AppState>,
    query: SearchQuery,
) -> Result<Vec<ComponentSummary>, String> {
    state.library.search(&query).await
}

#[tauri::command]
pub async fn get_component_detail(
    state: State<'_, AppState>,
    id: ComponentId,
) -> Result<Option<Component>, String> {
    state.library.get_by_id(id).await
}

#[tauri::command]
pub async fn create_component(
    state: State<'_, AppState>,
    name: String,
    description: String,
    manufacturer: Option<String>,
    category: Option<String>,
) -> Result<ComponentId, String> {
    let id = ComponentId(uuid::Uuid::new_v4());
    let now = chrono::Utc::now().to_rfc3339();
    let component = Component {
        id,
        name,
        description,
        manufacturer,
        category,
        symbol: SymbolDef::default(),
        footprint: FootprintDef::default(),
        model_3d: None,
        parameters: std::collections::HashMap::new(),
        created_at: now.clone(),
        updated_at: now,
    };
    state.library.save(&component).await?;
    Ok(id)
}

#[tauri::command]
pub async fn delete_component(
    state: State<'_, AppState>,
    id: ComponentId,
) -> Result<(), String> {
    state.library.delete(id).await
}

#[tauri::command]
pub async fn get_categories(
    state: State<'_, AppState>,
) -> Result<Vec<Category>, String> {
    state.library.get_categories().await
}

#[tauri::command]
pub async fn import_library_file(
    state: State<'_, AppState>,
    path: String,
    format: String,
) -> Result<usize, String> {
    let components = match format.as_str() {
        "kicad_sym" => crate::io::library_import::import_kicad_sym_lib(&path)?,
        "csv_bom" => crate::io::library_import::import_csv_bom(&path)?,
        "kicad_mod" | "kicad_pretty" => crate::io::library_import::import_kicad_pretty_lib(&path)?,
        _ => return Err(format!("Unsupported format: {}", format)),
    };
    let count = components.len();
    for comp in &components {
        state.library.save(comp).await?;
    }
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        conn.execute_batch("INSERT INTO components_fts(components_fts) VALUES('rebuild')")
            .map_err(|e| e.to_string())?;
    }
    Ok(count)
}

#[tauri::command]
pub async fn get_components_model_3d(
    state: State<'_, AppState>,
    ids: Vec<ComponentId>,
) -> Result<HashMap<String, Option<Model3D>>, String> {
    let mut result = HashMap::new();
    for id in ids {
        if let Some(component) = state.library.get_by_id(id).await? {
            result.insert(id.0.to_string(), component.model_3d);
        }
    }
    Ok(result)
}

#[tauri::command]
pub async fn save_symbol_def(
    state: State<'_, AppState>,
    component_id: ComponentId,
    symbol: SymbolDef,
) -> Result<(), String> {
    let mut component = state.library.get_by_id(component_id)
        .await?
        .ok_or_else(|| format!("Component not found: {}", component_id.0))?;
    component.symbol = symbol;
    component.updated_at = chrono::Utc::now().to_rfc3339();
    state.library.save(&component).await
}

#[tauri::command]
pub async fn get_footprint_defs(
    state: State<'_, AppState>,
    ids: Vec<ComponentId>,
) -> Result<HashMap<String, FootprintDef>, String> {
    let mut result = HashMap::new();
    for id in ids {
        if let Some(component) = state.library.get_by_id(id).await? {
            result.insert(id.0.to_string(), component.footprint);
        }
    }
    Ok(result)
}

#[tauri::command]
pub async fn save_footprint_def(
    state: State<'_, AppState>,
    component_id: ComponentId,
    footprint: FootprintDef,
) -> Result<(), String> {
    let mut component = state.library.get_by_id(component_id)
        .await?
        .ok_or_else(|| format!("Component not found: {}", component_id))?;
    component.footprint = footprint;
    component.updated_at = chrono::Utc::now().to_rfc3339();
    state.library.save(&component).await?;
    Ok(())
}

#[tauri::command]
pub async fn import_stackup_file(state: State<'_, AppState>) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let app_handle = {
        let guard = state.app_handle.read();
        guard.clone().ok_or("No app handle")?
    };
    let file = app_handle.dialog()
        .file()
        .add_filter("KiCad PCB", &["kicad_pcb", "kicad_sch"])
        .add_filter("All Files", &["*"])
        .blocking_pick_file();
    Ok(file.map(|p| match p {
        tauri_plugin_dialog::FilePath::Path(path) => path.to_string_lossy().to_string(),
        tauri_plugin_dialog::FilePath::Url(url) => url.to_string(),
    }))
}

#[tauri::command]
pub async fn parse_kicad_stackup(path: String) -> Result<LayerStack, String> {
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    
    // Parse KiCad PCB file for stackup info
    // This is a simplified parser for the (setup (stackup ...)) section
    let mut layers = Vec::new();
    let mut dielectrics = Vec::new();
    let copper_weight_oz = 1.0;
    
    // Simple regex-based extraction for common KiCad stackup format
    let re_layer = regex::Regex::new(r#"\(layer\s+\"(.*?)\".*?\(type\s+(\w+)\).*?\(color\s+#([0-9A-Fa-f]+)\)"#).unwrap();
    let re_dielectric = regex::Regex::new(r#"\(dielectric\s+\"(.*?)\".*?\(thickness\s+([0-9.]+)\).*?\(epsilon_r\s+([0-9.]+)\).*?\(loss_tangent\s+([0-9.]+)\)"#).unwrap();
    
    let mut layer_id_counter = 1;
    
    for cap in re_layer.captures_iter(&content) {
        let name = cap.get(1).map_or("", |m| m.as_str()).to_string();
        let kind = cap.get(2).map_or("Inner", |m| m.as_str());
        let color = cap.get(3).map_or("#D47A2F".to_string(), |m| format!("#{}", m.as_str()));
        layers.push(LayerDef {
            id: LayerId(layer_id_counter),
            name,
            kind: match kind {
                "copper" | "signal" => LayerKind::Inner,
                "soldermask" | "solder_mask" => LayerKind::SolderMask,
                "silkscreen" | "silk" => LayerKind::Silkscreen,
                "paste" => LayerKind::Paste,
                _ => LayerKind::Inner,
            },
            color,
        });
        layer_id_counter += 1;
    }
    
    for cap in re_dielectric.captures_iter(&content) {
        let name = cap.get(1).map_or("FR-4", |m| m.as_str()).to_string();
        let thickness = cap.get(2).map_or("0.1", |m| m.as_str()).parse().unwrap_or(0.1);
        let er = cap.get(3).map_or("4.5", |m| m.as_str()).parse().unwrap_or(4.5);
        let loss_tangent = cap.get(4).map_or("0.02", |m| m.as_str()).parse().unwrap_or(0.02);
        dielectrics.push(Dielectric { name, er, thickness_mm: thickness, loss_tangent });
    }
    
    // Fallback if parsing failed
    if layers.is_empty() {
        layers.push(LayerDef { id: LayerId(1), name: "Top".into(), kind: LayerKind::Top, color: "#D47A2F".into() });
        layers.push(LayerDef { id: LayerId(2), name: "Bottom".into(), kind: LayerKind::Bottom, color: "#A0793F".into() });
        layers.push(LayerDef { id: LayerId(10), name: "Top Solder Mask".into(), kind: LayerKind::SolderMask, color: "#30853A".into() });
        layers.push(LayerDef { id: LayerId(11), name: "Bottom Solder Mask".into(), kind: LayerKind::SolderMask, color: "#25682D".into() });
        layers.push(LayerDef { id: LayerId(20), name: "Top Silkscreen".into(), kind: LayerKind::Silkscreen, color: "#F0F0F0".into() });
        layers.push(LayerDef { id: LayerId(21), name: "Bottom Silkscreen".into(), kind: LayerKind::Silkscreen, color: "#C0C0C0".into() });
        dielectrics.push(Dielectric { name: "FR-4".into(), er: 4.5, thickness_mm: 1.6, loss_tangent: 0.02 });
    }
    
    Ok(LayerStack { layers, dielectrics, copper_weight_oz })
}
