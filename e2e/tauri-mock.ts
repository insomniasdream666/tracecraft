import { Page } from '@playwright/test';

export async function setupTauriMock(page: Page) {
  await page.addInitScript(() => {
    const listeners = new Map<string, Set<(...args: any[]) => void>>();

    const invoke = async (cmd: string, args?: any) => {
      switch (cmd) {
        case 'create_project':
          return { id: 1, name: args?.name ?? 'Untitled', path: `/tmp/${args?.name ?? 'Untitled'}.tcproj` };
        case 'get_schematic':
          return {
            id: 1,
            project_id: 1,
            name: 'Sheet 1',
            symbols: {},
            wires: {},
            net_labels: {},
            junctions: {},
            buses: {},
          };
        case 'place_symbol':
          return null;
        case 'draw_wire':
          return null;
        case 'add_net_label':
          return null;
        case 'add_junction':
          return null;
        case 'delete_element':
          return null;
        case 'annotate_schematic':
          return null;
        case 'run_erc_command':
          return [];
        case 'undo_schematic':
          return null;
        case 'redo_schematic':
          return null;
        case 'get_board_state':
          return {
            layers: {
              layers: [
                { id: 1, name: 'Top Copper', color: '#D47A2F' },
                { id: 2, name: 'Bottom Copper', color: '#5096E0' },
                { id: 10, name: 'Top Silkscreen', color: '#FFFFFF' },
                { id: 11, name: 'Bottom Silkscreen', color: '#AAAAAA' },
                { id: 20, name: 'Top Mask', color: '#1B5E20' },
                { id: 21, name: 'Bottom Mask', color: '#1B5E20' },
              ],
            },
            footprints: {},
            traces: {},
            vias: {},
            zones: {},
            outline: null,
          };
        case 'place_footprint':
          return null;
        case 'route_trace':
          return null;
        case 'add_via':
          return null;
        case 'draw_zone':
          return null;
        case 'delete_pcb_element':
          return null;
        case 'run_drc':
          return [];
        case 'autoroute_all':
          return null;
        case 'cancel_routing':
          return null;
        case 'sync_netlist':
          return null;
        case 'get_ratsnest':
          return [];
        case 'undo_pcb':
          return null;
        case 'redo_pcb':
          return null;
        case 'get_footprint_defs':
          return {};
        case 'get_board_bounds':
          return { x: -50, y: -50, width: 100, height: 100 };
        case 'get_schematic_bounds':
          return { x: -50, y: -50, width: 100, height: 100 };
        case 'set_trace_width':
        case 'set_trace_layer':
        case 'set_trace_net':
        case 'set_footprint_refdes':
        case 'move_footprint':
        case 'rotate_footprint':
        case 'move_via':
        case 'set_via_diameter':
        case 'set_via_drill':
        case 'set_via_net':
        case 'set_board_outline':
        case 'close_project':
          return null;
        case 'get_components_model_3d':
          return {};
        default:
          return null;
      }
    };

    const dialogPlugin = {
      open: async (options?: any) => '/tmp/mock-path.tcproj',
      save: async (options?: any) => '/tmp/mock-save.tcproj',
    };

    const fsPlugin = {
      writeFile: async (path: string, contents: Uint8Array) => {},
      writeTextFile: async (path: string, contents: string) => {},
    };

    const eventPlugin = {
      listen: async (event: string, handler: (payload: any) => void) => {
        if (!listeners.has(event)) listeners.set(event, new Set());
        listeners.get(event)!.add(handler);
        return () => listeners.get(event)?.delete(handler);
      },
      emit: async (event: string, payload?: any) => {
        listeners.get(event)?.forEach((h) => h({ payload }));
      },
    };

    window.__TAURI__ = {
      core: { invoke },
      plugin: {
        dialog: dialogPlugin,
        fs: fsPlugin,
        event: eventPlugin,
      },
    };

    Object.assign(window, {
      __TAURI_INTERNALS__: {
        invoke,
        convertFileSrc: (path: string) => path,
      },
    });
  });
}
