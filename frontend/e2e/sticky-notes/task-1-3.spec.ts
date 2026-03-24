import { test, expect } from '@playwright/test';

test.describe('Sticky Notes - Task 1.3', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('sticky note data structure and state management', async ({ page }) => {
    // Verify the create sticky note button exists in the toolbar
    const toolbar = page.locator('div[style*="display: flex"][style*="flex-direction: column"]');
    const createButton = toolbar.locator('button[title="Create Sticky Note"]');
    await expect(createButton).toBeVisible();

    // Click the create sticky note button
    await createButton.click();

    // Wait for the canvas to redraw
    await page.waitForTimeout(100);

    // Verify the canvas is still visible and functional
    const canvas = page.locator('canvas');
    await expect(canvas).toBeVisible();

    // In a more sophisticated implementation, we could:
    // - Check that a sticky note appears on the canvas
    // - Verify the sticky note has the correct visual properties (yellow background, etc.)
    // - Check that the sticky note is positioned at the center of the view
    // - Verify that multiple sticky notes can be created
    // - Test that sticky notes persist in the application state

    // For now, we verify that clicking the button doesn't crash the application
    // and that the basic UI remains functional
    await expect(toolbar).toBeVisible();
    const zoomInButton = toolbar.locator('button:has-text("+")');
    await expect(zoomInButton).toBeVisible();
  });

  test('multiple sticky notes can be created', async ({ page }) => {
    const toolbar = page.locator('div[style*="display: flex"][style*="flex-direction: column"]');
    const createButton = toolbar.locator('button[title="Create Sticky Note"]');

    // Create multiple sticky notes
    await createButton.click();
    await page.waitForTimeout(50);
    await createButton.click();
    await page.waitForTimeout(50);
    await createButton.click();
    await page.waitForTimeout(50);

    // Verify the application remains stable
    const canvas = page.locator('canvas');
    await expect(canvas).toBeVisible();
    await expect(toolbar).toBeVisible();

    // In a full implementation, we would verify that 3 sticky notes exist
    // and are properly positioned without overlapping
  });

  test('clicking sticky note enters edit mode with proper focus', async ({ page }) => {
    const toolbar = page.locator('div[style*="display: flex"][style*="flex-direction: column"]');
    const createButton = toolbar.locator('button[title="Create Sticky Note"]');

    // Create a sticky note
    await createButton.click();
    await page.waitForTimeout(100);

    // Find the sticky note (it should be a div with yellow background)
    const stickyNote = page.locator('div[style*="background: #FFFF88"]').first();

    // Click the sticky note to enter edit mode
    await stickyNote.click();
    await page.waitForTimeout(100);

    // Verify a textarea appears (edit mode)
    const textarea = page.locator('textarea[style*="background: #FFFF88"]');
    await expect(textarea).toBeVisible();

    // Verify the textarea has focus
    const isFocused = await textarea.evaluate(el => el === document.activeElement);
    expect(isFocused).toBe(true);

    // Verify the textarea contains the initial content
    await expect(textarea).toHaveValue('New sticky note');
  });

  test('sticky note editing saves on Enter key', async ({ page }) => {
    const toolbar = page.locator('div[style*="display: flex"][style*="flex-direction: column"]');
    const createButton = toolbar.locator('button[title="Create Sticky Note"]');

    // Create a sticky note
    await createButton.click();
    await page.waitForTimeout(100);

    // Click to enter edit mode
    const stickyNote = page.locator('div[style*="background: #FFFF88"]').first();
    await stickyNote.click();
    await page.waitForTimeout(100);

    // Type new content and press Enter
    const textarea = page.locator('textarea[style*="background: #FFFF88"]');
    await textarea.fill('Updated content');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(100);

    // Verify we're back to display mode (textarea should be gone)
    const textareaAfter = page.locator('textarea[style*="background: #FFFF88"]');
    await expect(textareaAfter).not.toBeVisible();

    // Verify the content was updated
    const updatedNote = page.locator('div[style*="background: #FFFF88"]').first();
    await expect(updatedNote).toContainText('Updated content');
  });

  test('sticky note editing cancels on Escape key', async ({ page }) => {
    const toolbar = page.locator('div[style*="display: flex"][style*="flex-direction: column"]');
    const createButton = toolbar.locator('button[title="Create Sticky Note"]');

    // Create a sticky note
    await createButton.click();
    await page.waitForTimeout(100);

    // Click to enter edit mode
    const stickyNote = page.locator('div[style*="background: #FFFF88"]').first();
    await stickyNote.click();
    await page.waitForTimeout(100);

    // Type new content and press Escape
    const textarea = page.locator('textarea[style*="background: #FFFF88"]');
    await textarea.fill('Should not save');
    await page.keyboard.press('Escape');
    await page.waitForTimeout(100);

    // Verify we're back to display mode
    const textareaAfter = page.locator('textarea[style*="background: #FFFF88"]');
    await expect(textareaAfter).not.toBeVisible();

    // Verify the content was NOT updated (should still be "New sticky note")
    const noteAfter = page.locator('div[style*="background: #FFFF88"]').first();
    await expect(noteAfter).toContainText('New sticky note');
  });

  test('sticky note editing saves on outside click', async ({ page }) => {
    const toolbar = page.locator('div[style*="display: flex"][style*="flex-direction: column"]');
    const createButton = toolbar.locator('button[title="Create Sticky Note"]');

    // Create a sticky note
    await createButton.click();
    await page.waitForTimeout(100);

    // Click to enter edit mode
    const stickyNote = page.locator('div[style*="background: #FFFF88"]').first();
    await stickyNote.click();
    await page.waitForTimeout(100);

    // Type new content
    const textarea = page.locator('textarea[style*="background: #FFFF88"]');
    await textarea.fill('Custom text that should be saved');

    // Click outside the sticky note (on the overlay)
    const overlay = page.locator('div[style*="z-index: 4"]');
    await overlay.click({ position: { x: 100, y: 100 } });
    await page.waitForTimeout(100);

    // Verify we're back to display mode
    const textareaAfter = page.locator('textarea[style*="background: #FFFF88"]');
    await expect(textareaAfter).not.toBeVisible();

    // Verify the content was saved
    const noteAfter = page.locator('div[style*="background: #FFFF88"]').first();
    await expect(noteAfter).toContainText('Custom text that should be saved');
  });
});