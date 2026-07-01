import { Point, Rotation, Size } from "../lib/geometry";

export interface SchematicSheet {
  id: number;
  name: string;
  symbols: Record<number, SymbolInstance>;
  wires: Record<number, Wire>;
  junctions: Record<number, Junction>;
  net_labels: Record<number, NetLabel>;
  buses: Record<number, Bus>;
  text_annotations: Record<number, TextAnnotation>;
  hier_sheets: Record<number, HierarchicalSheet>;
  properties: Record<string, string>;
}

export interface SymbolInstance {
  component_id: string;
  position: Point;
  rotation: Rotation;
  mirrored: boolean;
  refdes: string;
  value: string;
}

export interface Wire {
  points: Point[];
  net_id: number | null;
  width: number;
}

export interface Junction {
  position: Point;
}

export interface NetLabel {
  name: string;
  position: Point;
}

export interface TextAnnotation {
  text: string;
  position: Point;
  size: number;
}

export interface Bus {
  name: string;
  members: string[];
  wires: Point[][];
}

export interface HierarchicalSheet {
  id: number;
  name: string;
  child_sheet_id: number;
  pins: HierarchicalPin[];
  position: Point;
  size: Size;
}

export interface HierarchicalPin {
  name: string;
  pin_number: string;
  position: Point;
}
