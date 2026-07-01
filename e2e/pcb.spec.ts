import { test, expect } from '@playwright/test';
import { setupTauriMock } from './tauri-mock';

test.describe('PCB editor', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page);
    await page.goto('/');
  });

  test('switch to PCB view', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'PCB' }).click();
    const pcbCanvas = page.locator('canvas').first();
    await expect(pcbCanvas).toBeVisible({ timeout: 10000 });
  });

  test('PCB canvas renders after switching', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'PCB' }).click();
    await expect(page.locator('button:has-text("Run DRC")')).toBeVisible();
  });

  test('PCB tools are available in toolbar', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'PCB' }).click();
    await expect(page.locator('button').filter({ hasText: 'Route' })).toBeVisible();
    await expect(page.locator('button').filter({ hasText: 'Zone' })).toBeVisible();
    await expect(page.locator('button').filter({ hasText: 'Board Outline' })).toBeVisible();
  });

  test('PCB-specific toolbar buttons appear', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'PCB' }).click();
    await expect(page.locator('button').filter({ has: page.locator('.lucide-refresh-cw') })).toBeVisible();
    await expect(page.locator('button').filter({ has: page.locator('.lucide-network') })).toBeVisible();
  });

  test('status bar updates mode to PCB Layout', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'PCB' }).click();
    await expect(page.locator('text=PCB Layout')).toBeVisible();
  });

  test('empty board shows placeholder text', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'PCB' }).click();
    const placeholder = page.locator('text=Drag components from the library to place them on the board');
    await expect(placeholder).toBeVisible({ timeout: 5000 });
  });

  test('Run DRC button exists on PCB canvas', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'PCB' }).click();
    await expect(page.locator('button:has-text("Run DRC")')).toBeVisible();
  });
});
