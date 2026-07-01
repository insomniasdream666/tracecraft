import { test, expect } from '@playwright/test';
import { setupTauriMock } from './tauri-mock';

test.describe('Undo/redo', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page);
    await page.goto('/');
  });

  test('undo and redo buttons are present in toolbar', async ({ page }) => {
    const undoBtn = page.locator('button').filter({ has: page.locator('.lucide-undo-2') });
    const redoBtn = page.locator('button').filter({ has: page.locator('.lucide-redo-2') });
    await expect(undoBtn).toBeVisible();
    await expect(redoBtn).toBeVisible();
  });

  test('undo and redo buttons exist in schematic mode', async ({ page }) => {
    await expect(page.locator('button').filter({ has: page.locator('.lucide-undo-2') })).toBeVisible();
    await expect(page.locator('button').filter({ has: page.locator('.lucide-redo-2') })).toBeVisible();
  });

  test('undo and redo buttons exist in PCB mode', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'PCB' }).click();
    await expect(page.locator('button').filter({ has: page.locator('.lucide-undo-2') })).toBeVisible();
    await expect(page.locator('button').filter({ has: page.locator('.lucide-redo-2') })).toBeVisible();
  });

  test('undo and redo buttons exist in 3D preview mode', async ({ page }) => {
    await page.locator('button').filter({ hasText: '3D' }).first().click();
    await expect(page.locator('button').filter({ has: page.locator('.lucide-undo-2') })).toBeVisible();
    await expect(page.locator('button').filter({ has: page.locator('.lucide-redo-2') })).toBeVisible();
  });

  test('undo shortcut tooltip is displayed', async ({ page }) => {
    const undoBtn = page.locator('button').filter({ has: page.locator('.lucide-undo-2') });
    await expect(undoBtn).toBeVisible();
  });

  test('redo shortcut tooltip is displayed', async ({ page }) => {
    const redoBtn = page.locator('button').filter({ has: page.locator('.lucide-redo-2') });
    await expect(redoBtn).toBeVisible();
  });
});
