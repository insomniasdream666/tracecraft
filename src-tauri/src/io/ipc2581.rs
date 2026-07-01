use crate::model::pcb::PcbBoard;
use quick_xml::Writer;
use quick_xml::events::{Event, BytesDecl, BytesEnd, BytesStart};

pub fn export_ipc2581(board: &PcbBoard, path: &str) -> Result<(), String> {
    let mut writer = Writer::new_with_indent(
        std::io::BufWriter::new(std::fs::File::create(path).map_err(|e| e.to_string())?),
        b' ', 2,
    );
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None))).map_err(|e| e.to_string())?;

    let root = BytesStart::new("IPC-2581").with_attributes(vec![
        ("xmlns", "http://www.ipc.org/2581"),
        ("version", "3.0"),
        ("units", "mm"),
    ]);
    writer.write_event(Event::Start(root)).map_err(|e| e.to_string())?;

    writer.write_event(
        Event::Start(BytesStart::new("Design").with_attributes(vec![("name", "TraceCraft")]))
    ).map_err(|e| e.to_string())?;

    writer.write_event(Event::Start(BytesStart::new("Board"))).map_err(|e| e.to_string())?;
    writer.write_event(Event::Start(BytesStart::new("LayerStack"))).map_err(|e| e.to_string())?;

    for (name, layer_type) in [("F.Cu", "signal"), ("B.Cu", "signal"), ("Edge.Cuts", "outline")] {
        let el = BytesStart::new("Layer").with_attributes(vec![
            ("name", name),
            ("type", layer_type),
            ("thickness", "0.035"),
        ]);
        writer.write_event(Event::Empty(el)).map_err(|e| e.to_string())?;
    }

    writer.write_event(Event::End(BytesEnd::new("LayerStack"))).map_err(|e| e.to_string())?;

    writer.write_event(Event::Start(BytesStart::new("NetList"))).map_err(|e| e.to_string())?;
    for net in &board.netlist.nets {
        let el = BytesStart::new("Net").with_attributes(vec![
            ("name", net.name.as_str()),
            ("net_class", "default"),
        ]);
        writer.write_event(Event::Empty(el)).map_err(|e| e.to_string())?;
    }
    writer.write_event(Event::End(BytesEnd::new("NetList"))).map_err(|e| e.to_string())?;

    writer.write_event(Event::Start(BytesStart::new("TraceList"))).map_err(|e| e.to_string())?;
    for (_, trace) in board.traces.iter() {
        if trace.points.len() >= 2 {
            let layer_name = match trace.layer.0 { 2 => "B.Cu", _ => "F.Cu" };
            writer.write_event(
                Event::Start(BytesStart::new("Trace").with_attributes(vec![("layer", layer_name)]))
            ).map_err(|e| e.to_string())?;
            for pt in &trace.points {
                let xs = format!("{:.6}", pt.x);
                let ys = format!("{:.6}", pt.y);
                let el = BytesStart::new("Point").with_attributes(vec![
                    ("x", xs.as_str()),
                    ("y", ys.as_str()),
                ]);
                writer.write_event(Event::Empty(el)).map_err(|e| e.to_string())?;
            }
            writer.write_event(Event::End(BytesEnd::new("Trace"))).map_err(|e| e.to_string())?;
        }
    }
    writer.write_event(Event::End(BytesEnd::new("TraceList"))).map_err(|e| e.to_string())?;

    writer.write_event(Event::Start(BytesStart::new("ComponentList"))).map_err(|e| e.to_string())?;
    for (_, fp) in board.footprints.iter() {
        let el = BytesStart::new("Component").with_attributes(vec![
            ("name", fp.refdes.as_str()),
        ]);
        writer.write_event(Event::Empty(el)).map_err(|e| e.to_string())?;
    }
    writer.write_event(Event::End(BytesEnd::new("ComponentList"))).map_err(|e| e.to_string())?;

    writer.write_event(Event::End(BytesEnd::new("Board"))).map_err(|e| e.to_string())?;
    writer.write_event(Event::End(BytesEnd::new("Design"))).map_err(|e| e.to_string())?;
    writer.write_event(Event::End(BytesEnd::new("IPC-2581"))).map_err(|e| e.to_string())?;
    Ok(())
}
