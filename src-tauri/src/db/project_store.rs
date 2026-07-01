use rusqlite::{params, Connection};
use crate::model::project::*;
use uuid::Uuid;

pub fn save_project(conn: &Connection, project: &Project) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS project_meta (
            id TEXT PRIMARY KEY, name TEXT NOT NULL,
            created_at TEXT NOT NULL, updated_at TEXT NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO project_meta (id, name, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![project.id.to_string(), project.name, project.created_at, project.updated_at],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn load_project(conn: &Connection, id: Uuid) -> Result<Option<Project>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, name, '', created_at, updated_at FROM project_meta WHERE id = ?1"
    ).map_err(|e| e.to_string())?;

    let mut rows = stmt.query_map(params![id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let uuid = uuid::Uuid::parse_str(&id_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        Ok(Project {
            id: uuid,
            name: row.get(1)?,
            path: String::new(),
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.next().transpose().map_err(|e| e.to_string())
}
