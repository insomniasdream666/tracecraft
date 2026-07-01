export interface Component {
  id: string;
  name: string;
  description: string;
  manufacturer: string | null;
  category: string | null;
  symbol: SymbolDef;
  footprint: FootprintDef;
  model_3d: Model3D | null;
  parameters: Record<string, string>;
  created_at: string;
  updated_at: string;
}

export interface SymbolDef {
  pins: PinDef[];
  body: SymbolGraphics[];
  origin: { x: number; y: number };
}

export interface PinDef {
  name: string;
  number: string;
  position: { x: number; y: number };
  length: number;
  electrical_type: PinType;
}

export type PinType = "Input" | "Output" | "Bidirectional" | "Power" | "Gnd" | "Passive" | "OpenCollector" | "OpenEmitter" | "NotConnected";

export type SymbolGraphics = 
  | { type: "Rectangle"; position: { x: number; y: number }; size: { width: number; height: number }; fill: string; stroke: string }
  | { type: "Circle"; center: { x: number; y: number }; radius: number; fill: string; stroke: string }
  | { type: "Arc"; center: { x: number; y: number }; radius: number; start_angle: number; end_angle: number; stroke: string }
  | { type: "Line"; start: { x: number; y: number }; end: { x: number; y: number }; stroke: string; width: number }
  | { type: "Text"; text: string; position: { x: number; y: number }; size: number };

export interface FootprintDef {
  pads: PadDef[];
  outline: { x: number; y: number }[];
  courtyard: { x: number; y: number }[];
}

export interface PadDef {
  number: string;
  shape: PadShape;
  position: { x: number; y: number };
  size: { width: number; height: number };
  layer: number;
  drill: number | null;
}

export type PadShape = "Round" | "Rect" | "Oval" | "RoundRect" | "Custom";

export interface Model3D {
  path: string;
  offset: { x: number; y: number };
  rotation: number;
}

export interface ComponentSummary {
  id: string;
  name: string;
  description: string;
  category: string | null;
  manufacturer: string | null;
}

export interface Category {
  id: string;
  name: string;
  parent_id: string | null;
}

export interface SearchQuery {
  text: string;
  category?: string;
  page: number;
  per_page: number;
}
