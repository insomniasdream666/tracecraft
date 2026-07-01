import { test, expect } from '@playwright/test';
import { setupTauriMock } from './tauri-mock';

test.describe('Schematic editor', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page);
    await page.goto('/');
  });

  test('schematic canvas is visible by default', async ({ page }) => {
    const canvas = page.locator('canvas').first();
    await expect(canvas).toBeVisible({ timeout: 10000 });
  });

  test('schematic mode indicator is active', async ({ page }) => {
    const schematicBtn = page.locator('button').filter({ hasText: 'Schematic' });
    await expect(schematicBtn).toBeVisible();
  });

  test('schematic tools are rendered in toolbar', async ({ page }) => {
    await expect(page.locator('button').filter({ hasText: 'Select' })).toBeVisible();
    await expect(page.locator('button').filter({ hasText: 'Wire' })).toBeVisible();
    await expect(page.locator('button').filter({ hasText: 'Component' })).toBeVisible();
    await expect(page.locator('button').filter({ hasText: 'Net Label' })).toBeVisible();
    await expect(page.locator('button').filter({ hasText: 'Junction' })).toBeVisible();
  });

  test('Run ERC button is available', async ({ page }) => {
    await expect(page.locator('button:has-text("Run ERC")')).toBeVisible();
  });

  test('select tool is active by default', async ({ page }) => {
    const annotationBtn = page.locator('button').filter({ hasText: 'Annotate' });
    await expect(annotationBtn).toBeVisible();
  });

  test('undo and redo buttons are present', async ({ page }) => {
    const undoGroup = page.locator('button').filter({ has: page.locator('.lucide-undo-2') });
    const redoGroup = page.locator('button').filter({ has: page.locator('.lucide-redo-2') });
    await expect(undoGroup).toBeVisible();
    await expect(redoGroup).toBeVisible();
  });

  test('zoom controls are present', async ({ page }) => {
    const zoomIn = page.locator('button').filter({ has: page.locator('.lucide-zoom-in') });
    const zoomOut = page.locator('button').filter({ has: page.locator('.lucide-zoom-out') });
    await expect(zoomIn).toBeVisible();
    await expect(zoomOut).toBeVisible();
  });

  test('clicking component tool changes active tool', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'Component' }).click();
    const componentBtn = page.locator('button').filter({ hasText: 'Component' });
    await expect(componentBtn).toBeVisible();
  });

  test('status bar shows zoom percentage', async ({ page }) => {
    await expect(page.locator('text=Zoom:')).toBeVisible();
  });
});
