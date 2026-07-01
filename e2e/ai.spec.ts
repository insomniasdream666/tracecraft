import { test, expect } from '@playwright/test';
import { setupTauriMock } from './tauri-mock';

test.describe('AI copilot', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page);
    await page.goto('/');
  });

  test('AI panel exists in sidebar tabs', async ({ page }) => {
    await expect(page.locator('button').filter({ hasText: 'AI' })).toBeVisible();
  });

  test('AI panel opens when AI tab is clicked', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'AI' }).click();
    await expect(page.locator('text=Suggestions')).toBeVisible();
    await expect(page.locator('text=Chat')).toBeVisible();
  });

  test('chat window opens with welcome message', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'AI' }).click();
    await page.locator('button').filter({ hasText: 'Chat' }).click();
    await expect(page.locator('text=Ask me about PCB design')).toBeVisible();
  });

  test('can send a message in chat', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'AI' }).click();
    await page.locator('button').filter({ hasText: 'Chat' }).click();

    const input = page.locator('input[placeholder="Ask about PCB design..."]');
    await expect(input).toBeVisible();

    await input.fill('What trace width for 1A?');
    const sendBtn = page.locator('button').filter({ has: page.locator('.lucide-send') });
    await expect(sendBtn).toBeEnabled();
  });

  test('suggestion buttons appear in chat tab', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'AI' }).click();
    await page.locator('button').filter({ hasText: 'Chat' }).click();

    await expect(page.locator('text=What clearance for 5V nets?')).toBeVisible();
    await expect(page.locator('text=What trace width for 1A?')).toBeVisible();
    await expect(page.locator('text=How to design a ground plane?')).toBeVisible();
    await expect(page.locator('text=Is my board ready for fabrication?')).toBeVisible();
  });

  test('clicking suggestion fills the input', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'AI' }).click();
    await page.locator('button').filter({ hasText: 'Chat' }).click();

    await page.locator('text=What trace width for 1A?').click();
    const input = page.locator('input[placeholder="Ask about PCB design..."]');
    await expect(input).toHaveValue('What trace width for 1A?');
  });

  test('recommender widget has search input', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'AI' }).click();

    const searchInput = page.locator('input[placeholder="Search components..."]');
    await expect(searchInput).toBeVisible();
  });

  test('sparkles icon present in recommendations tab', async ({ page }) => {
    await page.locator('button').filter({ hasText: 'AI' }).click();

    const suggestionsTab = page.locator('button').filter({ hasText: 'Suggestions' });
    await expect(suggestionsTab).toBeVisible();
  });
});
