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
    await this.canvas.waitFor({ state: 'visible' });
    const boundingBox = await this.canvas.boundingBox();
    if (!boundingBox || boundingBox.width <= 0 || boundingBox.height <= 0) {
      throw new Error('Canvas does not have valid dimensions');
    }
  }

  // Actions
  async zoomIn() {
    await this.zoomInButton.click();
  }

  async zoomOut() {
    await this.zoomOutButton.click();
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