use rusqlite::Connection;

const MIGRATIONS: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS components (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT NOT NULL DEFAULT '',
        manufacturer TEXT,
        category TEXT,
        symbol_json TEXT NOT NULL DEFAULT '{}',
        footprint_json TEXT NOT NULL DEFAULT '{}',
        model_3d_json TEXT,
        parameters_json TEXT NOT NULL DEFAULT '{}',
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS categories (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        parent_id TEXT REFERENCES categories(id)
    );
    CREATE VIRTUAL TABLE IF NOT EXISTS components_fts USING fts5(
        name, description, manufacturer, category,
        content='components',
        content_rowid='rowid'
    );
    INSERT INTO categories (id, name) VALUES
        ('resistor', 'Resistors'),
        ('capacitor', 'Capacitors'),
        ('inductor', 'Inductors'),
        ('diode', 'Diodes'),
        ('transistor', 'Transistors'),
        ('ic', 'ICs'),
        ('connector', 'Connectors'),
        ('power', 'Power'),
        ('sensor', 'Sensors'),
        ('electromechanical', 'Electromechanical'),
        ('misc', 'Miscellaneous')
    ON CONFLICT(name) DO NOTHING;",
    "CREATE TRIGGER IF NOT EXISTS components_ai AFTER INSERT ON components BEGIN
        INSERT INTO components_fts(rowid, name, description, manufacturer, category)
        VALUES (new.rowid, new.name, new.description, new.manufacturer, new.category);
    END;
    CREATE TRIGGER IF NOT EXISTS components_ad AFTER DELETE ON components BEGIN
        INSERT INTO components_fts(components_fts, rowid, name, description, manufacturer, category)
        VALUES ('delete', old.rowid, old.name, old.description, old.manufacturer, old.category);
    END;
    CREATE TRIGGER IF NOT EXISTS components_au AFTER UPDATE ON components BEGIN
        INSERT INTO components_fts(components_fts, rowid, name, description, manufacturer, category)
        VALUES ('delete', old.rowid, old.name, old.description, old.manufacturer, old.category);
        INSERT INTO components_fts(rowid, name, description, manufacturer, category)
        VALUES (new.rowid, new.name, new.description, new.manufacturer, new.category);
    END;",
    "INSERT INTO categories (id, name) VALUES ('rf', 'RF') ON CONFLICT(name) DO NOTHING;",
];

pub fn run_migrations(conn: &Connection) -> Result<(), String> {
    let version: i32 = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .map_err(|e| e.to_string())?;

    for (i, migration) in MIGRATIONS.iter().enumerate() {
        let task_version = (i + 1) as i32;
        if version < task_version {
            conn.execute_batch(migration).map_err(|e| e.to_string())?;
            conn.pragma_update(None, "user_version", task_version)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
