import { describe, it, expect, beforeEach, vi } from 'vitest';

const mockInvoke = vi.hoisted(() => vi.fn());

vi.mock('@tauri-apps/api/core', () => ({ invoke: mockInvoke }));
vi.mock('@tauri-apps/plugin-dialog', () => ({ save: vi.fn() }));
vi.mock('@tauri-apps/plugin-fs', () => ({ writeFile: vi.fn(), writeTextFile: vi.fn() }));

import { useSchematicStore } from '../schematicStore';

describe('schematicStore', () => {
  beforeEach(() => {
    useSchematicStore.setState({
      sheet: null,
      selectedIds: new Set(),
      loading: false,
      error: null,
    });
    mockInvoke.mockReset();
  });

  it('loadSheet calls invoke and sets state', async () => {
    const mockSheet = {
      id: 1,
      name: 'Sheet1',
      symbols: {}, wires: {}, junctions: {}, net_labels: {},
      buses: {}, text_annotations: {}, hier_sheets: {}, properties: {},
    };
    mockInvoke.mockResolvedValue(mockSheet);

    await useSchematicStore.getState().loadSheet(1);

    expect(mockInvoke).toHaveBeenCalledWith('get_schematic', { sheetId: 1 });
    expect(useSchematicStore.getState().sheet).toEqual(mockSheet);
    expect(useSchematicStore.getState().loading).toBe(false);
  });

  it('loadSheet sets error on failure', async () => {
    mockInvoke.mockRejectedValue('failed');

    await useSchematicStore.getState().loadSheet();

    expect(useSchematicStore.getState().error).toBe('failed');
    expect(useSchematicStore.getState().loading).toBe(false);
  });

  it('undo calls invoke and reloads sheet', async () => {
    const mockSheet = {
      id: 2, name: 'Sheet2',
      symbols: {}, wires: {}, junctions: {}, net_labels: {},
      buses: {}, text_annotations: {}, hier_sheets: {}, properties: {},
    };
    useSchematicStore.setState({ sheet: mockSheet });
    mockInvoke.mockResolvedValue(undefined);
    mockInvoke.mockResolvedValue(mockSheet);

    await useSchematicStore.getState().undo();

    expect(mockInvoke).toHaveBeenCalledWith('undo_schematic');
    expect(mockInvoke).toHaveBeenCalledWith('get_schematic', { sheetId: 2 });
  });

  it('redo calls invoke and reloads sheet', async () => {
    const mockSheet = {
      id: 2, name: 'Sheet2',
      symbols: {}, wires: {}, junctions: {}, net_labels: {},
      buses: {}, text_annotations: {}, hier_sheets: {}, properties: {},
    };
    useSchematicStore.setState({ sheet: mockSheet });
    mockInvoke.mockResolvedValue(undefined);
    mockInvoke.mockResolvedValue(mockSheet);

    await useSchematicStore.getState().redo();

    expect(mockInvoke).toHaveBeenCalledWith('redo_schematic');
    expect(mockInvoke).toHaveBeenCalledWith('get_schematic', { sheetId: 2 });
  });

  it('select replaces selection when add=false', () => {
    const { select } = useSchematicStore.getState();

    select('sym1');
    expect(useSchematicStore.getState().selectedIds).toEqual(new Set(['sym1']));

    select('sym2');
    expect(useSchematicStore.getState().selectedIds).toEqual(new Set(['sym2']));
  });

  it('select with add=true toggles ids in multi-select', () => {
    const { select } = useSchematicStore.getState();

    select('sym1');
    select('sym2', true);
    expect(useSchematicStore.getState().selectedIds).toEqual(new Set(['sym1', 'sym2']));

    select('sym1', true);
    expect(useSchematicStore.getState().selectedIds).toEqual(new Set(['sym2']));
  });
});
