import { test, expect } from './test-setup';

test('Zoom buttons increase/decrease zoom level', async ({ canvasPage }) => {
  // Get initial zoom level
  const initialZoom = await canvasPage.getZoomLevel();
  expect(initialZoom).toBe('1'); // Initial zoom should be 1.0

  // Verify zoom buttons exist
  await expect(canvasPage.zoomInButton).toBeVisible();
  await expect(canvasPage.zoomOutButton).toBeVisible();

  // Click zoom in button
  await canvasPage.zoomIn();

  // Wait for zoom to update and verify it increased
  expect(await canvasPage.getZoomLevel()).toBe('1.2');

  // Click zoom out button
  await canvasPage.zoomOut();

  // Wait for zoom to update and verify it decreased back to initial
  expect(await canvasPage.getZoomLevel()).toBe('1');
});