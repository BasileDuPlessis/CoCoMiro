import { test as base, expect } from '@playwright/test';
import { InfiniteCanvasPage } from './page-objects';

// Extend the base test with common setup
const test = base.extend<{
  canvasPage: InfiniteCanvasPage;
}>({
  canvasPage: async ({ page }, use) => {
    // Navigate to the application
    await page.goto('http://127.0.0.1:8080');

    // Create page object
    const canvasPage = new InfiniteCanvasPage(page);

    // Wait for canvas to be ready
    await canvasPage.waitForCanvasReady();

    // Pass the page object to the test
    await use(canvasPage);
  },
});

export { test, expect };