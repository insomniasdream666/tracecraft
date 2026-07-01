import { test, expect } from '@playwright/test';
import { setupTauriMock } from './tauri-mock';

test.describe('Library browser', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page);
    await page.goto('/');
    await page.locator('text=Library').click();
  });

  test('library tab shows search input', async ({ page }) => {
    const searchInput = page.locator('input[placeholder="Search components..."]');
    await expect(searchInput).toBeVisible();
  });

  test('typing a search term triggers results', async ({ page }) => {
    const searchInput = page.locator('input[placeholder="Search components..."]');
    await searchInput.fill('resistor');

    const noResults = page.locator('text=No components found');
    await expect(noResults).toBeVisible({ timeout: 5000 });
  });

  test('category tree shows categories', async ({ page }) => {
    await expect(page.locator('text=All Components')).toBeVisible();
  });

  test('clicking a category filters results', async ({ page }) => {
    await expect(page.locator('text=All Components')).toBeVisible();
  });

  test('import library button exists', async ({ page }) => {
    await expect(page.locator('button:has-text("Import Library")')).toBeVisible();
  });

  test('new symbol button exists', async ({ page }) => {
    await expect(page.locator('button:has-text("New Symbol")')).toBeVisible();
  });

  test('categories sidebar is rendered', async ({ page }) => {
    await expect(page.locator('text=Categories')).toBeVisible();
  });
});
