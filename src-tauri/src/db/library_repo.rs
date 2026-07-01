use rusqlite::{params, Connection};
use std::sync::Mutex;
use crate::model::library::*;

pub struct SqliteComponentRepository {
    conn: Mutex<Connection>,
}

impl SqliteComponentRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn: Mutex::new(conn) }
    }
}

#[async_trait::async_trait]
impl ComponentRepository for SqliteComponentRepository {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<ComponentSummary>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let offset = query.page * query.per_page;

        let mut stmt = if query.text.is_empty() {
            conn.prepare(
                "SELECT id, name, description, category, manufacturer
                 FROM components
                 WHERE (?1 IS NULL OR category = ?1)
                 ORDER BY name LIMIT ?2 OFFSET ?3"
            ).map_err(|e| e.to_string())?
        } else {
            conn.prepare(
                "SELECT c.id, c.name, c.description, c.category, c.manufacturer
                 FROM components c
                 JOIN components_fts fts ON c.rowid = fts.rowid
                 WHERE components_fts MATCH ?1
                 AND (?2 IS NULL OR c.category = ?2)
                 ORDER BY rank LIMIT ?3 OFFSET ?4"
            ).map_err(|e| e.to_string())?
        };

        let rows = if query.text.is_empty() {
            let mapped = stmt.query_map(
                params![query.category, query.per_page as i64, offset as i64],
                |row| {
                    let id_str: String = row.get(0)?;
                    Ok(ComponentSummary {
                        id: ComponentId(uuid::Uuid::parse_str(&id_str).unwrap_or(uuid::Uuid::nil())),
                        name: row.get(1)?,
                        description: row.get(2)?,
                        category: row.get(3)?,
                        manufacturer: row.get(4)?,
                    })
                },
            ).map_err(|e| e.to_string())?;
            mapped.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
        } else {
            let mapped = stmt.query_map(
                params![query.text, query.category, query.per_page as i64, offset as i64],
                |row| {
                    let id_str: String = row.get(0)?;
                    Ok(ComponentSummary {
                        id: ComponentId(uuid::Uuid::parse_str(&id_str).unwrap_or(uuid::Uuid::nil())),
                        name: row.get(1)?,
                        description: row.get(2)?,
                        category: row.get(3)?,
                        manufacturer: row.get(4)?,
                    })
                },
            ).map_err(|e| e.to_string())?;
            mapped.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
        };

        Ok(rows)
    }

    async fn get_by_id(&self, id: ComponentId) -> Result<Option<Component>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, manufacturer, category,
                    symbol_json, footprint_json, model_3d_json,
                    parameters_json, created_at, updated_at
             FROM components WHERE id = ?1"
        ).map_err(|e| e.to_string())?;

        let mut rows = stmt.query_map(params![id.0.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let uuid = uuid::Uuid::parse_str(&id_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            Ok(Component {
                id: ComponentId(uuid),
                name: row.get(1)?,
                description: row.get(2)?,
                manufacturer: row.get(3)?,
                category: row.get(4)?,
                symbol: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                footprint: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                model_3d: row.get::<_, Option<String>>(7)?.and_then(|s| serde_json::from_str(&s).ok()),
                parameters: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default(),
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }).map_err(|e| e.to_string())?;

        rows.next().transpose().map_err(|e| e.to_string())
    }

    async fn save(&self, component: &Component) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO components (id, name, description, manufacturer, category,
             symbol_json, footprint_json, model_3d_json, parameters_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
             name=excluded.name, description=excluded.description,
             manufacturer=excluded.manufacturer, category=excluded.category,
             symbol_json=excluded.symbol_json, footprint_json=excluded.footprint_json,
             model_3d_json=excluded.model_3d_json, parameters_json=excluded.parameters_json,
             updated_at=excluded.updated_at",
            params![
                component.id.0.to_string(),
                component.name,
                component.description,
                component.manufacturer,
                component.category,
                serde_json::to_string(&component.symbol).unwrap_or_default(),
                serde_json::to_string(&component.footprint).unwrap_or_default(),
                component.model_3d.as_ref().map(|m| serde_json::to_string(m).unwrap_or_default()),
                serde_json::to_string(&component.parameters).unwrap_or_default(),
                now, now,
            ],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn delete(&self, id: ComponentId) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM components WHERE id = ?1", params![id.0.to_string()])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn get_categories(&self) -> Result<Vec<Category>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, name, parent_id FROM categories ORDER BY name")
            .map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
            })
        }).map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }
}
