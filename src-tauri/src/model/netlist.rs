use serde::{Deserialize, Serialize};
use crate::model::id::NetId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Netlist {
    pub nets: Vec<Net>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Net {
    pub id: NetId,
    pub name: String,
    pub connections: Vec<PinRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PinRef {
    pub refdes: String,
    pub pin_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetlistDiff {
    pub added: Vec<Net>,
    pub removed: Vec<Net>,
    pub modified: Vec<(String, Net, Net)>,
}
