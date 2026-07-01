import { describe, it, expect, beforeEach, vi } from 'vitest';

const mockInvoke = vi.hoisted(() => vi.fn());

vi.mock('@tauri-apps/api/core', () => ({ invoke: mockInvoke }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));

import { usePCBStore } from '../pcbStore';

describe('pcbStore', () => {
  beforeEach(() => {
    usePCBStore.setState({
      board: null,
      selectedIds: new Set(),
      activeLayer: 1,
      visibleLayers: new Set([1, 2, 10, 11, 20, 21]),
      loading: false,
      error: null,
      ratsnest: [],
      drcViolations: [],
      footprintCache: {},
      routingActive: false,
      routingProgress: null,
    });
    mockInvoke.mockReset();
  });

  it('loadBoard calls invoke and sets state', async () => {
    const mockBoard = {
      footprints: {},
      traces: {},
      vias: {},
      zones: {},
      outline: null,
      layers: { layers: [] },
      design_rules: {
        min_trace_width: 0.2, min_clearance: 0.2, min_annular_ring: 0.15,
        min_hole_size: 0.3, default_trace_width: 0.25, default_via_diameter: 0.6, default_via_drill: 0.3,
      },
      netlist: { nets: [] },
    };
    mockInvoke.mockResolvedValue(mockBoard);

    await usePCBStore.getState().loadBoard();

    expect(mockInvoke).toHaveBeenCalledWith('get_board_state');
    expect(usePCBStore.getState().board).toEqual(mockBoard);
    expect(usePCBStore.getState().loading).toBe(false);
  });

  it('loadBoard sets error on failure', async () => {
    mockInvoke.mockRejectedValue('load failed');

    await usePCBStore.getState().loadBoard();

    expect(usePCBStore.getState().error).toBe('load failed');
    expect(usePCBStore.getState().loading).toBe(false);
  });

  it('placeFootprint calls invoke with correct args', async () => {
    mockInvoke.mockResolvedValue(undefined);

    await usePCBStore.getState().placeFootprint('comp1', 10, 20);

    expect(mockInvoke).toHaveBeenCalledWith('place_footprint', {
      componentId: 'comp1',
      position: { x: 10, y: 20 },
      rotation: 0,
    });
  });

  it('select replaces selection when add=false', () => {
    const { select } = usePCBStore.getState();

    select('fp1');
    expect(usePCBStore.getState().selectedIds).toEqual(new Set(['fp1']));

    select('fp2');
    expect(usePCBStore.getState().selectedIds).toEqual(new Set(['fp2']));
  });

  it('select with add=true toggles ids in multi-select', () => {
    const { select } = usePCBStore.getState();

    select('fp1');
    select('fp2', true);
    expect(usePCBStore.getState().selectedIds).toEqual(new Set(['fp1', 'fp2']));

    select('fp1', true);
    expect(usePCBStore.getState().selectedIds).toEqual(new Set(['fp2']));
  });

  it('setActiveLayer updates active layer', () => {
    usePCBStore.getState().setActiveLayer(3);
    expect(usePCBStore.getState().activeLayer).toBe(3);
  });

  it('undo and redo call invoke', async () => {
    mockInvoke.mockResolvedValue(undefined);
    const mockBoard = {
      footprints: {}, traces: {}, vias: {}, zones: {}, outline: null,
      layers: { layers: [] },
      design_rules: { min_trace_width: 0.2, min_clearance: 0.2, min_annular_ring: 0.15, min_hole_size: 0.3, default_trace_width: 0.25, default_via_diameter: 0.6, default_via_drill: 0.3 },
      netlist: { nets: [] },
    };
    mockInvoke.mockResolvedValue(mockBoard);

    await usePCBStore.getState().undo();
    expect(mockInvoke).toHaveBeenCalledWith('undo_pcb');

    await usePCBStore.getState().redo();
    expect(mockInvoke).toHaveBeenCalledWith('redo_pcb');
  });
});
