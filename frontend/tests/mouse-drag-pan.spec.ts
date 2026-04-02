import { test, expect } from './test-setup';

test('Mouse drag panning moves canvas', async ({ canvasPage }) => {
  // Get initial pan position
  const initialPanX = await canvasPage.getPanX();
  const initialPanY = await canvasPage.getPanY();
  expect(initialPanX).toBe('0'); // Initial pan should be 0
  expect(initialPanY).toBe('0');

  // Get canvas bounding box for drag coordinates
  const canvasBox = await canvasPage.canvas.boundingBox();
  expect(canvasBox).not.toBeNull();

  // Calculate drag coordinates (start from center, drag 100px right and down)
  const startX = canvasBox!.x + canvasBox!.width / 2;
  const startY = canvasBox!.y + canvasBox!.height / 2;
  const endX = startX + 100;
  const endY = startY + 100;

  // Perform drag operation
  await canvasPage.dragCanvas(startX, startY, endX, endY);

  // Wait for pan position to change (instead of fixed timeout)
  await expect(async () => {
    const currentPanX = await canvasPage.getPanX();
    const currentPanY = await canvasPage.getPanY();
    expect(parseFloat(currentPanX!)).toBeGreaterThan(0);
    expect(parseFloat(currentPanY!)).toBeGreaterThan(0);
  }).toPass({ timeout: 1000 });

  // Get final pan position for delta calculation
  const finalPanX = await canvasPage.getPanX();
  const finalPanY = await canvasPage.getPanY();

  // Verify the drag distance is approximately correct
  // Note: The actual pan values depend on zoom level and implementation details
  const deltaX = parseFloat(finalPanX!) - parseFloat(initialPanX!);
  const deltaY = parseFloat(finalPanY!) - parseFloat(initialPanY!);
  expect(deltaX).toBeCloseTo(100, -1); // Allow some tolerance
  expect(deltaY).toBeCloseTo(100, -1);
});