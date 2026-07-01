use crate::model::pcb::PcbBoard;
use crate::model::schematic::SchematicSheet;

#[derive(Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

pub fn template_chat(message: &str, _board: Option<&PcbBoard>, _schematics: Option<&SchematicSheet>) -> Result<String, String> {
    let lower = message.to_lowercase();

    let reply = if lower.contains("clearance") || lower.contains("spacing") {
        if lower.contains("5v") || lower.contains("power") {
            "For power nets (e.g. 5V), a clearance of 0.25mm is recommended. For high-voltage (>50V), use >= 1.0mm.".to_string()
        } else {
            "Standard signal nets typically use 0.15mm clearance. Check design rules in your PCB settings.".to_string()
        }
    } else if lower.contains("trace") || lower.contains("width") || lower.contains("track") {
        if lower.contains("power") || lower.contains("high current") {
            "For power traces, width depends on current. 1A ~ 0.25mm on 1oz copper. Use a trace width calculator for precise values.".to_string()
        } else {
            "Standard signal traces are typically 0.25mm wide. Your design rules set a minimum of 0.15mm.".to_string()
        }
    } else if lower.contains("via") || lower.contains("drill") || lower.contains("hole") {
        "Standard via: 0.6mm diameter, 0.3mm drill. For high-density boards, micro-vias (0.2mm drill) are an option.".to_string()
    } else if lower.contains("rout") || lower.contains("autorout") || lower.contains("auto rout") {
        "Try the auto-router from the PCB view's Route menu. It uses A* pathfinding with clearance checking.".to_string()
    } else if lower.contains("ground") || lower.contains("gnd") || lower.contains("plane") {
        "Use a ground plane on an inner layer for best EMI performance. You can draw a zone and assign it to the GND net.".to_string()
    } else if lower.contains("layer") || lower.contains("stack") {
        "A 2-layer stack: Top (signal) + Bottom (ground plane). For complex designs, 4-layer: Top-GND-VCC-Bottom.".to_string()
    } else if lower.contains("thank") || lower.contains("great") || lower.contains("help") {
        "You're welcome! Let me know if you need help with routing, placement, or design rules.".to_string()
    } else if lower.contains("hello") || lower.contains("hi ") || lower.contains("hey") {
        "Hello! I'm your PCB design assistant. Ask me about trace widths, clearances, vias, routing, or ground planes.".to_string()
    } else if lower.contains("board ready") || lower.contains("fabricat") || lower.contains("manufactur") {
        "Before sending to fab: check all design rules, run ERC on schematic, run DRC on PCB, verify netlist sync, and generate Gerber files.".to_string()
    } else if lower.contains("error") || lower.contains("erc") || lower.contains("drc") {
        "Run ERC in schematic view to check for floating pins, unconnected nets. Run DRC in PCB view for clearance violations.".to_string()
    } else if lower.contains("net") || lower.contains("connect") {
        "Nets are created automatically when you place wires and net labels in the schematic. Keep net names descriptive (e.g. VCC_3V3, I2C_SCL).".to_string()
    } else {
        "I can help with: clearance rules, trace widths, vias, routing, ground planes, layer stackup, net naming, and preparing for fabrication. Try asking a specific question!".to_string()
    };

    Ok(reply)
}
