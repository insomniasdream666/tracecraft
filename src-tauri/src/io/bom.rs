use crate::model::pcb::PcbBoard;

pub fn generate_bom(board: &PcbBoard, path: &str) -> Result<(), String> {
    let mut wtr = csv::Writer::from_path(path).map_err(|e| e.to_string())?;
    wtr.write_record(["Reference", "Value", "Quantity"])
        .map_err(|e| e.to_string())?;

    for (_, fp) in board.footprints.iter() {
        wtr.write_record([&fp.refdes, "", "1"])
            .map_err(|e| e.to_string())?;
    }

    wtr.flush().map_err(|e| e.to_string())?;
    Ok(())
}
