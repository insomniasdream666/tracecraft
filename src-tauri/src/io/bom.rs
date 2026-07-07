use std::collections::BTreeMap;
use crate::model::pcb::PcbBoard;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct BomEntry {
    pub reference: String,
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
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (_, fp) in board.footprints.iter() {
        let cid = fp.component_id.to_string();
        groups.entry(cid).or_default().push(fp.refdes.clone());
    }

    let mut entries: Vec<BomEntry> = groups
        .into_iter()
        .map(|(cid, refs)| BomEntry {
            reference: refs.join(", "),
            component_id: cid,
            quantity: refs.len() as u32,
        })
        .collect();
    entries.sort_by(|a, b| natural_sort_key(&a.reference).cmp(&natural_sort_key(&b.reference)));

    let mut wtr = csv::Writer::from_path(path).map_err(|e| e.to_string())?;
    wtr.write_record(["Reference", "Component ID", "Quantity"])
        .map_err(|e| e.to_string())?;
    for e in &entries {
        wtr.write_record([&e.reference, &e.component_id, &e.quantity.to_string()])
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
