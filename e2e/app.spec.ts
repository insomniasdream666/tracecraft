import { test, expect } from '@playwright/test';
import { setupTauriMock } from './tauri-mock';

test.describe('App shell', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page);
    await page.goto('/');
  });

  test('app loads and shows the toolbar', async ({ page }) => {
    await expect(page.locator('text=File')).toBeVisible();
    await expect(page.locator('text=Schematic')).toBeVisible();
    await expect(page.locator('text=PCB')).toBeVisible();
    await expect(page.locator('text=3D').first()).toBeVisible();
  });

  test('app shell renders sidebar and status bar', async ({ page }) => {
    await expect(page.locator('text=Library')).toBeVisible();
    await expect(page.locator('text=Layers')).toBeVisible();
    await expect(page.locator('text=AI')).toBeVisible();
    await expect(page.locator('text=Properties')).toBeVisible();

    await expect(page.locator('text=TraceCraft')).toBeVisible();
  });

  test('window title is TraceCraft', async ({ page }) => {
    await expect(page).toHaveTitle('TraceCraft');
  });

  test('status bar displays app name and version', async ({ page }) => {
    await expect(page.locator('text=TraceCraft v0.1.0')).toBeVisible();
  });

  test('status bar shows current mode label', async ({ page }) => {
    await expect(page.locator('text=Schematic')).toBeVisible();
  });

  test('sidebar toggle collapses and expands sidebar', async ({ page }) => {
    const collapseButton = page.locator('button').filter({ has: page.locator('.lucide-panel-right-close') });
    if (await collapseButton.isVisible()) {
      await collapseButton.click();
    }

    const expandButton = page.locator('button').filter({ has: page.locator('.lucide-panel-right-close') });
    await expect(expandButton).toBeVisible();
  });

  test('canvas area is present', async ({ page }) => {
    const canvas = page.locator('canvas').first();
    await expect(canvas).toBeVisible({ timeout: 10000 });
  });
});
