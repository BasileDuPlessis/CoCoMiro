import { test, expect } from './test-setup';

test('Canvas loads and displays correctly', async ({ canvasPage }) => {
  // Canvas is already loaded and verified by the test setup
  // Additional checks can be added here if needed

  // Verify the canvas has the expected initial zoom level
  const initialZoom = await canvasPage.getZoomLevel();
  expect(initialZoom).toBe('1');
});