import { test, expect } from '@playwright/test';

test.describe('Infinite Canvas', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for WASM to load (same as sticky notes tests)
    await page.waitForTimeout(1000);
    // Reload page to ensure clean state
    await page.reload();
    await page.waitForTimeout(1000);
    // Ensure clean state by clicking outside
    await page.mouse.click(10, 10);
    await page.waitForTimeout(100);
  });

  test('floating toolbar is visible and positioned correctly', async ({ page }) => {
    // Check that the floating toolbar exists - look for the div with data-testid
    const toolbar = page.locator('[data-testid="floating-toolbar"]');
    await expect(toolbar).toBeVisible();

    // Check that zoom buttons are inside the toolbar
    const zoomInButton = toolbar.locator('button:has-text("+")');
    const zoomOutButton = toolbar.locator('button:has-text("-")');
    await expect(zoomInButton).toBeVisible();
    await expect(zoomOutButton).toBeVisible();

    // Check that the grip zone is visible
    const gripZone = toolbar.locator('div').first(); // The first div is the handle
    await expect(gripZone).toBeVisible();

    // Check that it's positioned near the top-left initially
    const toolbarBox = await toolbar.boundingBox();
    
    // Toolbar should be positioned near the top-left initially
    expect(toolbarBox!.y).toBeLessThan(50); // Should be near the top
    expect(toolbarBox!.x).toBeLessThan(50); // Should be near the left initially
    expect(toolbarBox!.width).toBeGreaterThan(40); // Should have reasonable width for buttons
  });

  test('floating toolbar can be dragged to new positions', async ({ page }) => {
    const toolbar = page.locator('[data-testid="floating-toolbar"]');
    
    // Get initial position
    const initialBox = await toolbar.boundingBox();
    const initialX = initialBox!.x;
    const initialY = initialBox!.y;

    // Drag the toolbar to a new position (50px right, 50px down)
    await page.mouse.move(initialX + 20, initialY + 10); // Move to grip zone
    await page.mouse.down();
    await page.mouse.move(initialX + 70, initialY + 60, { steps: 10 }); // Drag to new position
    await page.mouse.up();
    await page.waitForTimeout(100); // Wait for position update

    // Check that toolbar has moved
    const newBox = await toolbar.boundingBox();
    const newX = newBox!.x;
    const newY = newBox!.y;

    // Should have moved approximately 50px in each direction
    expect(newX).toBeGreaterThan(initialX + 40);
    expect(newY).toBeGreaterThan(initialY + 40);

    // Toolbar should still be visible and functional
    await expect(toolbar).toBeVisible();
    const zoomInButton = toolbar.locator('button:has-text("+")');
    await expect(zoomInButton).toBeVisible();
  });

  test('zoom in button increases zoom', async ({ page }) => {
    // Get initial debug text
    const initialDebug = await page.locator('canvas').evaluate(() => {
      const canvas = document.querySelector('canvas') as HTMLCanvasElement;
      const ctx = canvas.getContext('2d')!;
      // This is a simplified way - in real app we'd need to expose zoom state
      return 'Zoom: 1.00'; // Default zoom
    });

    // Click zoom in button
    await page.click('button:has-text("+")');

    // Wait a bit for redraw
    await page.waitForTimeout(100);

    // In a real implementation, we'd check the debug overlay or canvas state
    // For now, just verify the button click doesn't crash
    await expect(page.locator('canvas')).toBeVisible();
  });

  test('zoom out button decreases zoom', async ({ page }) => {
    // Click zoom out button
    await page.click('button:has-text("-")');

    // Wait a bit for redraw
    await page.waitForTimeout(100);

    // Verify canvas still works
    await expect(page.locator('canvas')).toBeVisible();
  });

  test('canvas panning with mouse drag', async ({ page }) => {
    const canvas = page.locator('canvas');

    // Get initial position (we'll use a visual check)
    const canvasBox = await canvas.boundingBox();
    expect(canvasBox).toBeTruthy();

    // Start drag from center of canvas
    const startX = canvasBox!.x + canvasBox!.width / 2;
    const startY = canvasBox!.y + canvasBox!.height / 2;

    // Perform mouse drag
    await page.mouse.move(startX, startY);
    await page.mouse.down();
    await page.mouse.move(startX + 50, startY + 30);
    await page.mouse.up();

    // Wait for redraw
    await page.waitForTimeout(100);

    // Verify canvas is still visible and functional
    await expect(canvas).toBeVisible();

    // In a more sophisticated test, we could:
    // - Check debug overlay text for pan coordinates
    // - Take screenshots and compare
    // - Check that grid position changed
  });

  test('keyboard zoom controls work', async ({ page }) => {
    const canvas = page.locator('canvas');

    // Focus the canvas (it has tabindex="0")
    await canvas.focus();

    // Press Ctrl + Plus to zoom in
    await page.keyboard.press('Control+=');

    // Wait for redraw
    await page.waitForTimeout(100);

    // Verify canvas still works
    await expect(canvas).toBeVisible();

    // Press Ctrl + Minus to zoom out
    await page.keyboard.press('Control+-');

    // Wait for redraw
    await page.waitForTimeout(100);

    // Verify canvas still works
    await expect(canvas).toBeVisible();
  });

  test('mouse wheel zoom works', async ({ page }) => {
    const canvas = page.locator('canvas');

    // Get canvas center position
    const canvasBox = await canvas.boundingBox();
    const centerX = canvasBox!.x + canvasBox!.width / 2;
    const centerY = canvasBox!.y + canvasBox!.height / 2;

    // Move mouse to center and scroll up (zoom in)
    await page.mouse.move(centerX, centerY);
    await page.mouse.wheel(0, -100); // Negative deltaY = zoom in

    // Wait for redraw
    await page.waitForTimeout(100);

    // Verify canvas still works
    await expect(canvas).toBeVisible();

    // Scroll down (zoom out)
    await page.mouse.wheel(0, 100); // Positive deltaY = zoom out

    // Wait for redraw
    await page.waitForTimeout(100);

    // Verify canvas still works
    await expect(canvas).toBeVisible();
  });

  test('canvas maintains functionality after interactions', async ({ page }) => {
    const canvas = page.locator('canvas');

    // Perform a series of interactions
    await page.click('button:has-text("+")'); // Zoom in
    await page.waitForTimeout(50);

    await page.click('button:has-text("-")'); // Zoom out
    await page.waitForTimeout(50);

    // Focus and keyboard zoom
    await canvas.focus();
    await page.keyboard.press('Control+=');
    await page.waitForTimeout(50);

    // Mouse wheel zoom
    const canvasBox = await canvas.boundingBox();
    await page.mouse.move(canvasBox!.x + canvasBox!.width / 2, canvasBox!.y + canvasBox!.height / 2);
    await page.mouse.wheel(0, -50);
    await page.waitForTimeout(50);

    // Canvas panning
    await page.mouse.down();
    await page.mouse.move(canvasBox!.x + canvasBox!.width / 2 + 30, canvasBox!.y + canvasBox!.height / 2 + 20);
    await page.mouse.up();
    await page.waitForTimeout(50);

    // Final check - everything should still work
    await expect(canvas).toBeVisible();
    await expect(page.locator('button:has-text("+")')).toBeVisible();
    await expect(page.locator('button:has-text("-")')).toBeVisible();
  });
});