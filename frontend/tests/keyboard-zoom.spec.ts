import { test, expect } from './test-setup';

test('Keyboard zoom shortcuts work correctly', async ({ canvasPage }) => {
  // Get initial zoom level
  const initialZoom = await canvasPage.getZoomLevel();
  expect(initialZoom).toBe('1'); // Initial zoom should be 1.0

  // Test Ctrl+Plus increases zoom
  await canvasPage.zoomInKeyboard();

  // Wait for zoom to update and verify it increased
  expect(await canvasPage.getZoomLevel()).toBe('1.2');

  // Test Ctrl+Minus decreases zoom back to initial
  await canvasPage.zoomOutKeyboard();

  // Wait for zoom to update and verify it decreased
  expect(await canvasPage.getZoomLevel()).toBe('1');

  // Test Ctrl+= (equals) also increases zoom
  await canvasPage.zoomInKeyboardEquals();

  // Wait for zoom to update and verify it increased
  expect(await canvasPage.getZoomLevel()).toBe('1.2');
});

test('Keyboard zoom respects minimum and maximum limits', async ({ canvasPage }) => {
  // Test maximum zoom limit (5.0)
  // Start from zoom 1.0 and zoom in repeatedly until we hit the limit
  // 1.0 * 1.2^7 = 3.5832, 1.0 * 1.2^8 = 4.2998, 1.0 * 1.2^9 = 5.1598 (clamped to 5.0)
  // So we need about 9 zoom ins to hit the limit
  for (let i = 0; i < 10; i++) {
    await canvasPage.zoomInKeyboard();
  }

  // Should be clamped to maximum zoom
  expect(await canvasPage.getZoomLevel()).toBe('5');

  // Test minimum zoom limit (0.1)
  // Reset by zooming out repeatedly
  // 5.0 / 1.2^20 ≈ 0.0779, 5.0 / 1.2^21 ≈ 0.0649, etc. down to 0.1
  // So we need about 21 zoom outs to hit the limit from max
  for (let i = 0; i < 25; i++) {
    await canvasPage.zoomOutKeyboard();
  }

  // Should be clamped to minimum zoom
  expect(await canvasPage.getZoomLevel()).toBe('0.1');
});