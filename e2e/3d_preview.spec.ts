import { test, expect } from '@playwright/test';
import { setupTauriMock } from './tauri-mock';

test.describe('3D preview', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page);
    await page.goto('/');
  });

  test('3D tab button exists in toolbar', async ({ page }) => {
    await expect(page.locator('button').filter({ hasText: '3D' }).first()).toBeVisible();
  });

  test('switch to 3D preview tab', async ({ page }) => {
    await page.locator('button').filter({ hasText: '3D' }).first().click();
    const canvas = page.locator('canvas').first();
    await expect(canvas).toBeVisible({ timeout: 10000 });
  });

  test('3D viewer renders canvas', async ({ page }) => {
    await page.locator('button').filter({ hasText: '3D' }).first().click();
    const threeCanvas = page.locator('canvas').first();
    await expect(threeCanvas).toBeVisible({ timeout: 10000 });
  });

  test('layer visibility toggle checkboxes appear', async ({ page }) => {
    await page.locator('button').filter({ hasText: '3D' }).first().click();
    await expect(page.locator('text=Copper')).toBeVisible();
    await expect(page.locator('text=Silkscreen')).toBeVisible();
  });

  test('Copper checkbox toggles visibility', async ({ page }) => {
    await page.locator('button').filter({ hasText: '3D' }).first().click();
    const copperCheckbox = page.locator('label').filter({ hasText: 'Copper' }).locator('input[type="checkbox"]');
    await expect(copperCheckbox).toBeVisible();
    await copperCheckbox.click();
    await expect(copperCheckbox).not.toBeChecked();
  });

  test('Silkscreen checkbox toggles visibility', async ({ page }) => {
    await page.locator('button').filter({ hasText: '3D' }).first().click();
    const silkscreenCheckbox = page.locator('label').filter({ hasText: 'Silkscreen' }).locator('input[type="checkbox"]');
    await expect(silkscreenCheckbox).toBeVisible();
    await silkscreenCheckbox.click();
    await expect(silkscreenCheckbox).not.toBeChecked();
  });

  test('Measure button appears in 3D view', async ({ page }) => {
    await page.locator('button').filter({ hasText: '3D' }).first().click();
    await expect(page.locator('button:has-text("Measure")')).toBeVisible();
  });

  test('status bar updates mode to 3D Preview', async ({ page }) => {
    await page.locator('button').filter({ hasText: '3D' }).first().click();
    await expect(page.locator('text=3D Preview')).toBeVisible();
  });
});
