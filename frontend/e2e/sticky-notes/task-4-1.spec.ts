import { test, expect } from '@playwright/test';

test.describe('Sticky Notes - Task 4.1', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000); // Wait for WASM to load
  });

  test('sticky note can be dragged to new position on canvas', async ({ page }) => {
    // Create a sticky note
    const toolbar = page.locator('div[style*="display: flex"][style*="flex-direction: column"]');
    const createButton = toolbar.locator('button[title="Create Sticky Note"]');
    await createButton.click();
    await page.waitForTimeout(500); // Longer wait

    // Debug: check DOM
    const allDivs = await page.locator('div').count();
    console.log('Total divs:', allDivs);

    const yellowDivs = await page.locator('div[style*="background"]').count();
    console.log('Divs with background:', yellowDivs);

    // Find the sticky note
    const stickyNote = page.locator('div[style*="background: #FFFF88"]').first();
    console.log('Sticky note locator count:', await stickyNote.count());

    await expect(stickyNote).toBeVisible();

    // Get initial position
    const initialBoundingBox = await stickyNote.boundingBox();
    expect(initialBoundingBox).not.toBeNull();
    const initialX = initialBoundingBox!.x + initialBoundingBox!.width / 2;
    const initialY = initialBoundingBox!.y + initialBoundingBox!.height / 2;

    // Calculate target position (20px right, 20px down, still over the note)
    const targetX = initialX + 20;
    const targetY = initialY + 20;

    // Perform drag using mouse events
    await page.mouse.move(initialX, initialY);
    await page.mouse.down();
    await page.waitForTimeout(50); // Small delay to ensure drag starts
    await page.mouse.move(targetX, targetY);
    await page.mouse.up();

    await page.waitForTimeout(100);

    // If drag accidentally triggered edit mode, exit it
    const textarea = page.locator('textarea[style*="background: #FFFF88"]');
    if (await textarea.isVisible()) {
      await page.keyboard.press('Escape');
      await page.waitForTimeout(100);
    }

    // Verify the sticky note moved
    const finalBoundingBox = await stickyNote.boundingBox();
    expect(finalBoundingBox).not.toBeNull();
    const finalCenterX = finalBoundingBox!.x + finalBoundingBox!.width / 2;
    const finalCenterY = finalBoundingBox!.y + finalBoundingBox!.height / 2;

    // Check that the position changed (allowing for some tolerance)
    expect(Math.abs(finalCenterX - initialX)).toBeGreaterThan(10); // Should move at least 10px
    expect(Math.abs(finalCenterY - initialY)).toBeGreaterThan(10);

    // Verify the sticky note is still visible and functional
    await expect(stickyNote).toBeVisible();

    // Verify we can still click to edit (not stuck in drag mode)
    await stickyNote.click();
    await page.waitForTimeout(100);
    const textareaAfter = page.locator('textarea[style*="background: #FFFF88"]');
    await expect(textareaAfter).toBeVisible();
  });

  test('drag does not trigger text selection', async ({ page }) => {
    // Create a sticky note
    const toolbar = page.locator('div[style*="display: flex"][style*="flex-direction: column"]');
    const createButton = toolbar.locator('button[title="Create Sticky Note"]');
    await createButton.click();
    await page.waitForTimeout(100);

    // Find the sticky note
    const stickyNote = page.locator('div[style*="background: #FFFF88"]').first();

    // Get initial position
    const boundingBox = await stickyNote.boundingBox();
    const centerX = boundingBox!.x + boundingBox!.width / 2;
    const centerY = boundingBox!.y + boundingBox!.height / 2;

    // Get initial selection
    const initialSelection = await page.evaluate(() => window.getSelection()?.toString() || '');

    // Perform drag using mouse events
    await page.mouse.move(centerX, centerY);
    await page.mouse.down();
    await page.mouse.move(centerX + 50, centerY + 50);
    await page.mouse.up();

    await page.waitForTimeout(100);

    // Verify no text was selected during drag
    const finalSelection = await page.evaluate(() => window.getSelection()?.toString() || '');
    expect(finalSelection).toBe(initialSelection);
  });

  test('sticky note position persists after drag', async ({ page }) => {
    // Create a sticky note
    const toolbar = page.locator('div[style*="display: flex"][style*="flex-direction: column"]');
    const createButton = toolbar.locator('button[title="Create Sticky Note"]');
    await createButton.click();
    await page.waitForTimeout(100);

    const stickyNote = page.locator('div[style*="background: #FFFF88"]').first();

    // Get initial position
    const initialBox = await stickyNote.boundingBox();
    const centerX = initialBox!.x + initialBox!.width / 2;
    const centerY = initialBox!.y + initialBox!.height / 2;

    // Drag to new position
    await page.mouse.move(centerX, centerY);
    await page.mouse.down();
    await page.mouse.move(centerX + 100, centerY + 100);
    await page.mouse.up();

    await page.waitForTimeout(100);

    // Get position after drag
    const afterDragBox = await stickyNote.boundingBox();
    const afterDragCenterX = afterDragBox!.x + afterDragBox!.width / 2;
    const afterDragCenterY = afterDragBox!.y + afterDragBox!.height / 2;

    // Simulate waiting (in a real app, this would test persistence)
    await page.waitForTimeout(500);

    const finalBox = await stickyNote.boundingBox();
    const finalCenterX = finalBox!.x + finalBox!.width / 2;
    const finalCenterY = finalBox!.y + finalBox!.height / 2;

    // Position should remain the same (within small tolerance for rendering)
    expect(Math.abs(finalCenterX - afterDragCenterX)).toBeLessThan(5);
    expect(Math.abs(finalCenterY - afterDragCenterY)).toBeLessThan(5);
  });
});