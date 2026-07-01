use tauri::State;
use crate::app_state::AppState;
use crate::model::library::*;
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
