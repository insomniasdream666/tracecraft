pub mod ai;
pub mod app_state;
pub mod commands;
pub mod db;
pub mod engine;
pub mod io;
pub mod model;
pub mod undo;

use app_state::AppState;
use db::migrations;
use db::library_repo::SqliteComponentRepository;
use model::library::ComponentRepository;
use std::sync::Arc;
use tauri::Manager;

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tracecraft=debug".into()),
        )
        .init();

    let db_path = dirs_or_default();
    std::fs::create_dir_all(&db_path).ok();
    tracing::info!("TraceCraft starting, data dir: {}", db_path);

    let db_path = std::path::Path::new(&db_path).join("tracecraft.db");
    let conn = rusqlite::Connection::open(&db_path).expect("Failed to open database");
    migrations::run_migrations(&conn).expect("Failed to run migrations");
    if let Err(e) = db::seed::seed_components(&conn) {
        tracing::warn!("Failed to seed components: {}", e);
    }

    let db_for_state = conn;

    let library: Arc<dyn ComponentRepository> = Arc::new(
        SqliteComponentRepository::new(
            rusqlite::Connection::open(&db_path).expect("Failed to open library DB"),
        ),
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::new(db_for_state, library))
        .invoke_handler(tauri::generate_handler![
            commands::project::create_project,
            commands::project::save_project,
            commands::project::open_project,
            commands::project::close_project,
            commands::project::get_last_project,
            commands::schematic::get_schematic,
            commands::schematic::place_symbol,
            commands::schematic::draw_wire,
            commands::schematic::delete_element,
            commands::schematic::add_net_label,
            commands::schematic::add_junction,
            commands::schematic::run_erc_command,
            commands::schematic::annotate_schematic,
            commands::schematic::create_bus,
            commands::schematic::create_hierarchical_sheet,
            commands::schematic::move_elements,
            commands::pcb::get_board_state,
            commands::pcb::get_board_properties,
            commands::pcb::set_board_size,
            commands::pcb::place_footprint,
            commands::pcb::route_trace,
            commands::pcb::add_via,
            commands::pcb::set_board_outline,
            commands::pcb::sync_netlist,
            commands::pcb::sync_pcb_to_schematic,
            commands::pcb::get_ratsnest,
            commands::pcb::delete_pcb_element,
            commands::pcb::draw_zone,
            commands::pcb::get_3d_model_data,
            commands::pcb::create_diff_pair,
            commands::pcb::list_diff_pairs,
            commands::pcb::delete_diff_pair,
            commands::pcb::route_diff_pair,
            commands::pcb::get_stackup,
            commands::pcb::set_stackup,
            commands::pcb::compute_impedance,
            commands::pcb::apply_length_tuning,
            commands::library::search_components,
            commands::library::get_components_model_3d,
            commands::library::get_component_detail,
            commands::library::create_component,
            commands::library::delete_component,
            commands::library::get_categories,
            commands::library::import_library_file,
            commands::library::get_footprint_defs,
            commands::library::save_symbol_def,
            commands::library::save_footprint_def,
            commands::routing::autoroute_net,
            commands::routing::autoroute_all,
            commands::routing::fanout_component,
            commands::routing::cancel_routing,
            commands::drc::run_drc,
            commands::drc::get_drc_settings,
            commands::drc::update_drc_settings,
            commands::ai::ai_chat,
            commands::ai::recommend_components,
            commands::ai::suggest_route,
            commands::viewport::get_board_bounds,
            commands::viewport::get_schematic_bounds,
            commands::pcb::undo_pcb,
            commands::pcb::redo_pcb,
            commands::schematic::undo_schematic,
            commands::schematic::redo_schematic,
            commands::project::import_kicad_schematic,
            commands::project::export_kicad_schematic,
            commands::project::import_kicad_pcb,
            commands::project::export_kicad_pcb,
            commands::project::export_gerber,
            commands::project::export_ipc2581,
            commands::project::export_bom,
            commands::project::export_cpl,
            commands::project::export_spice_netlist,
            commands::library_download::download_kicad_libraries,
        ])
        .setup(|app| {
            let handle = app.handle();
            let state: tauri::State<AppState> = handle.state();
            *state.app_handle.write() = Some(handle.clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn dirs_or_default() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    format!("{}/.tracecraft", home)
}
