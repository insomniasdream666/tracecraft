use std::collections::BTreeMap;
use crate::model::pcb::PcbBoard;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct BomEntry {
    pub reference: String,
    pub value: String,
    pub component_id: String,
    pub quantity: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CplEntry {
    pub reference: String,
    pub mid_x: f64,
    pub mid_y: f64,
    pub rotation: f64,
    pub side: String,
}

pub fn generate_bom(board: &PcbBoard, path: &str) -> Result<(), String> {
    let mut groups: BTreeMap<String, (String, Vec<String>)> = BTreeMap::new();
    for (_, fp) in board.footprints.iter() {
        let cid = fp.component_id.to_string();
        groups
            .entry(cid)
            .or_insert_with(|| (fp.value.clone(), Vec::new()))
            .1
            .push(fp.refdes.clone());
    }

    let mut entries: Vec<BomEntry> = groups
        .into_iter()
        .map(|(cid, (value, refs))| BomEntry {
            reference: refs.join(", "),
            value,
            component_id: cid,
            quantity: refs.len() as u32,
        })
        .collect();
    entries.sort_by(|a, b| natural_sort_key(&a.reference).cmp(&natural_sort_key(&b.reference)));

    let mut wtr = csv::Writer::from_path(path).map_err(|e| e.to_string())?;
    wtr.write_record(["Reference", "Value", "Component ID", "Quantity"])
        .map_err(|e| e.to_string())?;
    for e in &entries {
        wtr.write_record([&e.reference, &e.value, &e.component_id, &e.quantity.to_string()])
            .map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn generate_cpl(board: &PcbBoard, path: &str) -> Result<(), String> {
    let mut wtr = csv::Writer::from_path(path).map_err(|e| e.to_string())?;
    wtr.write_record(["Reference", "Mid X", "Mid Y", "Rotation", "Side"])
        .map_err(|e| e.to_string())?;

    for (_, fp) in board.footprints.iter() {
        let side = if fp.mirrored { "bottom" } else { "top" };
        wtr.write_record([
            &fp.refdes,
            &format!("{:.4}", fp.position.x),
            &format!("{:.4}", fp.position.y),
            &format!("{:.2}", fp.rotation.0),
            side,
        ]).map_err(|e| e.to_string())?;
    }

    wtr.flush().map_err(|e| e.to_string())?;
    Ok(())
}

fn natural_sort_key(s: &str) -> String {
    let mut key = String::new();
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            key.push_str(&format!("{:08}", ch.to_digit(10).unwrap_or(0)));
        } else {
            key.push(ch);
        }
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::geometry::Point;
    use crate::model::id::ComponentId;
    use crate::model::pcb::FootprintInstance;

    #[test]
    fn bom_includes_value_and_groups_refs() {
        let mut board = PcbBoard::default();
        let cid = ComponentId(uuid::Uuid::new_v4());
        for (refdes, x) in [("R1", 0.0), ("R2", 10.0)] {
            board.footprints.insert(FootprintInstance {
                component_id: cid,
                position: Point::new(x, 0.0),
                rotation: crate::model::geometry::Rotation(0.0),
                mirrored: false,
                refdes: refdes.to_string(),
                value: "10k".to_string(),
                locked: false,
            });
        }
        let path = std::env::temp_dir().join("tracecraft-bom-values.csv");
        generate_bom(&board, path.to_str().unwrap()).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("Reference,Value,Component ID,Quantity"));
        assert!(text.contains("\"R1, R2\",10k"));
        let _ = std::fs::remove_file(path);
    }
}
