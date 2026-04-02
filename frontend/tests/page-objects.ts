// Page Object for the Infinite Canvas application
export class InfiniteCanvasPage {
  constructor(protected page: any) {}

  // Canvas element
  get canvas() {
    return this.page.locator('canvas');
  }

  // Toolbar elements
  get toolbar() {
    return this.page.locator('[data-testid="floating-toolbar"]');
  }

  get zoomInButton() {
    return this.toolbar.locator('button[title="Zoom In"]');
  }

  get zoomOutButton() {
    return this.toolbar.locator('button[title="Zoom Out"]');
  }

  get createStickyNoteButton() {
    return this.toolbar.locator('button[title="Create Sticky Note"]');
  }

  // Canvas state helpers
  async getZoomLevel() {
    return await this.canvas.getAttribute('data-zoom');
  }

  async getPanX() {
    return await this.canvas.getAttribute('data-pan-x');
  }

  async getPanY() {
    return await this.canvas.getAttribute('data-pan-y');
  }

  async waitForCanvasReady() {
    // First, wait for the page to load
    await this.page.waitForLoadState('domcontentloaded');

    // Check if the page has any content at all
    const bodyText = await this.page.locator('body').textContent();
    if (!bodyText || bodyText.trim() === '') {
      throw new Error('Page body is empty - application may not have loaded');
    }

    // Check if WASM loaded by looking for script tags or other indicators
    const scripts = await this.page.locator('script').count();
    if (scripts === 0) {
      throw new Error('No scripts found - WASM may not have loaded');
    }

    // Now check for canvas
    const canvasCount = await this.page.locator('canvas').count();
    if (canvasCount === 0) {
      // Wait a bit more for Yew to mount
      await this.page.waitForTimeout(3000);
      const canvasCount2 = await this.page.locator('canvas').count();
      if (canvasCount2 === 0) {
        // Get page content for debugging
        const html = await this.page.content();
        throw new Error(`Canvas element not found in DOM after waiting. Page HTML length: ${html.length}. Check if Yew app mounted properly.`);
      }
    }

    // Wait for canvas to be visible
    await this.canvas.waitFor({ state: 'visible', timeout: 5000 });

    // Wait for canvas to have proper dimensions
    const boundingBox = await this.canvas.boundingBox();
    if (!boundingBox || boundingBox.width <= 0 || boundingBox.height <= 0) {
      throw new Error('Canvas does not have valid dimensions');
    }

    // Wait for application initialization
    await this.page.waitForFunction(`() => {
      const canvas = document.querySelector('canvas');
      if (!canvas) return false;

      const loading = canvas.getAttribute('data-loading');
      if (loading === 'true') return false;

      const zoom = canvas.getAttribute('data-zoom');
      return zoom !== null && !isNaN(parseFloat(zoom));
    }`, { timeout: 8000 });
  }

  // Actions
  async zoomIn() {
    await this.zoomInButton.click();
  }

  async zoomOut() {
    await this.zoomOutButton.click();
  }

  async zoomInKeyboard() {
    await this.canvas.focus();
    await this.page.keyboard.press('Control++');
  }

  async zoomInKeyboardEquals() {
    await this.canvas.focus();
    await this.page.keyboard.press('Control+=');
  }

  async zoomOutKeyboard() {
    await this.canvas.focus();
    await this.page.keyboard.press('Control+-');
  }

  async createStickyNote() {
    await this.createStickyNoteButton.click();
  }

  async dragCanvas(fromX: number, fromY: number, toX: number, toY: number) {
    await this.page.mouse.move(fromX, fromY);
    await this.page.mouse.down();
    await this.page.mouse.move(toX, toY);
    await this.page.mouse.up();
  }
}