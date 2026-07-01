import { describe, it, expect, beforeEach, vi } from 'vitest';

const mockInvoke = vi.hoisted(() => vi.fn());

vi.mock('@tauri-apps/api/core', () => ({ invoke: mockInvoke }));

import { useLibraryStore } from '../libraryStore';

describe('libraryStore', () => {
  beforeEach(() => {
    useLibraryStore.setState({
      results: [],
      selected: null,
      categories: [],
      selectedCategory: null,
      query: '',
      loading: false,
      detailLoading: false,
      error: null,
      editingFootprintId: null,
    });
    mockInvoke.mockReset();
  });

  it('search updates results', async () => {
    const mockResults = [
      { id: '1', name: 'Resistor', description: 'A resistor', category: null, manufacturer: null },
    ];
    mockInvoke.mockResolvedValue(mockResults);

    await useLibraryStore.getState().search('resistor');

    expect(mockInvoke).toHaveBeenCalledWith('search_components', {
      query: { text: 'resistor', page: 0, per_page: 20 },
    });
    expect(useLibraryStore.getState().results).toEqual(mockResults);
    expect(useLibraryStore.getState().query).toBe('resistor');
  });

  it('search sets error on failure', async () => {
    mockInvoke.mockRejectedValue('search error');

    await useLibraryStore.getState().search('bad');

    expect(useLibraryStore.getState().error).toBe('search error');
    expect(useLibraryStore.getState().loading).toBe(false);
  });

  it('setCategory updates selectedCategory and re-searches', async () => {
    mockInvoke.mockResolvedValue([]);

    useLibraryStore.setState({ query: 'test' });
    await useLibraryStore.getState().setCategory('passive');

    expect(useLibraryStore.getState().selectedCategory).toBe('passive');
    expect(mockInvoke).toHaveBeenCalledWith('search_components', {
      query: { text: 'test', category: 'passive', page: 0, per_page: 20 },
    });
  });

  it('setCategory with null clears filter', async () => {
    mockInvoke.mockResolvedValue([]);
    useLibraryStore.setState({ query: 'test', selectedCategory: 'passive' });

    await useLibraryStore.getState().setCategory(null);

    expect(useLibraryStore.getState().selectedCategory).toBeNull();
  });

  it('loadCategories fetches and stores categories', async () => {
    const categories = [{ id: 'cat1', name: 'Passive', parent_id: null }];
    mockInvoke.mockResolvedValue(categories);

    await useLibraryStore.getState().loadCategories();

    expect(mockInvoke).toHaveBeenCalledWith('get_categories');
    expect(useLibraryStore.getState().categories).toEqual(categories);
  });
});
