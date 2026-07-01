import { test, expect } from '@playwright/test';
import { setupTauriMock } from './tauri-mock';

test.describe('File I/O', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page);
    await page.goto('/');
  });

  test('File menu opens on click', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'File' }).click();
    await expect(page.locator('text=New Project')).toBeVisible();
    await expect(page.locator('text=Open Project')).toBeVisible();
  });

  test('File menu shows all expected items', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'File' }).click();

    await expect(page.locator('text=New Project')).toBeVisible();
    await expect(page.locator('text=Open Project')).toBeVisible();
    await expect(page.locator('text=Save Project')).toBeVisible();
    await expect(page.locator('text=Close Project')).toBeVisible();
    await expect(page.locator('text=Export Gerber')).toBeVisible();
    await expect(page.locator('text=Export BOM (CSV)')).toBeVisible();
  });

  test('import options are listed', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'File' }).click();

    await expect(page.locator('text=Import KiCad Schematic')).toBeVisible();
    await expect(page.locator('text=Import KiCad PCB')).toBeVisible();
    await expect(page.locator('text=Import Library (KiCad sym)')).toBeVisible();
  });

  test('export options are listed', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'File' }).click();

    await expect(page.locator('text=Export Schematic Image')).toBeVisible();
    await expect(page.locator('text=Export KiCad Schematic')).toBeVisible();
    await expect(page.locator('text=Export KiCad PCB')).toBeVisible();
    await expect(page.locator('text=Export Gerber')).toBeVisible();
    await expect(page.locator('text=Export IPC-2581')).toBeVisible();
    await expect(page.locator('text=Export BOM (CSV)')).toBeVisible();
  });

  test('Save Project is disabled when no project open', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'File' }).click();
    const saveBtn = page.locator('button:has-text("Save Project")');
    await expect(saveBtn).toBeDisabled();
  });

  test('Close Project is disabled when no project open', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'File' }).click();
    const closeBtn = page.locator('button:has-text("Close Project")');
    await expect(closeBtn).toBeDisabled();
  });

  test('File menu closes on outside click', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'File' }).click();
    await expect(page.locator('text=New Project')).toBeVisible();

    await page.locator('text=Schematic').first().click();
    await expect(page.locator('text=New Project')).not.toBeVisible();
  });
});
