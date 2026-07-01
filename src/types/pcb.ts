import { Point, Rotation, Size } from "../lib/geometry";

export interface Netlist {
  nets: Net[];
}

export interface Net {
  id: number;
  name: string;
  connections: PinRef[];
}

export interface PinRef {
  refdes: string;
  pin_number: string;
}

export interface RatsnestLine {
  net_id: number | null;
  start: Point;
  end: Point;
  connected: boolean;
}

export interface PcbBoard {
  footprints: Record<number, FootprintInstance>;
  traces: Record<number, Trace>;
  vias: Record<number, Via>;
  zones: Record<number, Zone>;
  outline: BoardOutline | null;
  layers: LayerStack;
  design_rules: DesignRules;
  netlist: Netlist;
}

export interface FootprintInstance {
  component_id: string;
  position: Point;
  rotation: Rotation;
  mirrored: boolean;
  refdes: string;
  locked: boolean;
}

export interface Trace {
  net_id: number | null;
  layer: number;
  width: number;
  points: Point[];
}

export interface Via {
  position: Point;
  start_layer: number;
  end_layer: number;
  diameter: number;
  drill: number;
  net_id: number | null;
}

export interface Zone {
  net_id: number | null;
  layer: number;
  outline: Point[];
  clearance: number;
  thermal_relief: boolean;
}

export interface BoardOutline {
  shape: Point[][];
  cutouts: Point[][];
}

export interface LayerStack {
  layers: LayerDef[];
}

export interface LayerDef {
  id: number;
  name: string;
  kind: LayerKind;
  color: string;
}

export type LayerKind = "Top" | "Bottom" | "Inner" | "SolderMask" | "Silkscreen" | "Paste";

export interface DesignRules {
  min_trace_width: number;
  min_clearance: number;
  min_annular_ring: number;
  min_hole_size: number;
  default_trace_width: number;
  default_via_diameter: number;
  default_via_drill: number;
}
