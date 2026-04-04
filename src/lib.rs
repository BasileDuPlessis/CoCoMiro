#[cfg(target_arch = "wasm32")]
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
#[cfg(target_arch = "wasm32")]
use web_sys::{
    CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement, KeyboardEvent, MouseEvent,
    WheelEvent, window,
};

#[cfg(any(test, target_arch = "wasm32"))]
const DEFAULT_ZOOM: f64 = 1.0;
#[cfg(any(test, target_arch = "wasm32"))]
const MIN_ZOOM: f64 = 0.5;
#[cfg(any(test, target_arch = "wasm32"))]
const MAX_ZOOM: f64 = 2.5;

#[cfg(target_arch = "wasm32")]
const ZOOM_STEP_FACTOR: f64 = 1.1;
#[cfg(target_arch = "wasm32")]
const KEYBOARD_PAN_STEP: f64 = 40.0;
#[cfg(target_arch = "wasm32")]
const FALLBACK_VIEWPORT_WIDTH: f64 = 1280.0;
#[cfg(target_arch = "wasm32")]
const FALLBACK_VIEWPORT_HEIGHT: f64 = 840.0;
#[cfg(target_arch = "wasm32")]
const CANVAS_HORIZONTAL_MARGIN: f64 = 32.0;
#[cfg(target_arch = "wasm32")]
const CANVAS_VERTICAL_MARGIN: f64 = 96.0;
#[cfg(target_arch = "wasm32")]
const MIN_CANVAS_EDGE: f64 = 320.0;
#[cfg(target_arch = "wasm32")]
const GRID_BASE_SPACING: f64 = 48.0;
#[cfg(target_arch = "wasm32")]
const GRID_MIN_SPACING: f64 = 24.0;
#[cfg(target_arch = "wasm32")]
const GRID_MAX_SPACING: f64 = 120.0;
#[cfg(target_arch = "wasm32")]
const STATUS_HELP_TEXT: &str = "Drag to pan, scroll to zoom, or use the arrow keys and +/-.";

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq)]
struct ViewportState {
    pan_x: f64,
    pan_y: f64,
    zoom: f64,
    is_dragging: bool,
    last_mouse_pos: Option<(f64, f64)>,
}

#[cfg(any(test, target_arch = "wasm32"))]
impl Default for ViewportState {
    fn default() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: DEFAULT_ZOOM,
            is_dragging: false,
            last_mouse_pos: None,
        }
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
impl ViewportState {
    fn start_drag(&mut self, x: f64, y: f64) {
        self.is_dragging = true;
        self.last_mouse_pos = Some((x, y));
    }

    fn drag_to(&mut self, x: f64, y: f64) -> bool {
        if !self.is_dragging {
            return false;
        }

        if let Some((last_x, last_y)) = self.last_mouse_pos {
            self.pan_by(x - last_x, y - last_y);
            self.last_mouse_pos = Some((x, y));
            return true;
        }

        self.last_mouse_pos = Some((x, y));
        false
    }

    fn pan_by(&mut self, delta_x: f64, delta_y: f64) {
        self.pan_x += delta_x;
        self.pan_y += delta_y;
    }

    fn end_drag(&mut self) {
        self.is_dragging = false;
        self.last_mouse_pos = None;
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    fn zoom_by(&mut self, factor: f64) {
        self.zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
    }

    fn world_point_at(
        &self,
        screen_x: f64,
        screen_y: f64,
        viewport_width: f64,
        viewport_height: f64,
    ) -> (f64, f64) {
        let center_x = viewport_width / 2.0;
        let center_y = viewport_height / 2.0;

        (
            (screen_x - center_x - self.pan_x) / self.zoom,
            (screen_y - center_y - self.pan_y) / self.zoom,
        )
    }

    fn zoom_at(
        &mut self,
        factor: f64,
        cursor_x: f64,
        cursor_y: f64,
        viewport_width: f64,
        viewport_height: f64,
    ) {
        // Preserve the world-space point under the cursor so wheel zoom feels anchored.
        let world_point = self.world_point_at(cursor_x, cursor_y, viewport_width, viewport_height);
        self.zoom_by(factor);

        let center_x = viewport_width / 2.0;
        let center_y = viewport_height / 2.0;
        self.pan_x = cursor_x - center_x - (world_point.0 * self.zoom);
        self.pan_y = cursor_y - center_y - (world_point.1 * self.zoom);
    }
}

#[cfg(target_arch = "wasm32")]
fn app_markup() -> String {
    r#"
        <main class="app-shell">
            <section class="canvas-panel">
                <p id="canvas-status" class="canvas-status" role="status" aria-live="polite">Pan (0, 0) · Zoom 1.00× · Drag to pan, scroll to zoom, or use the arrow keys and +/-.</p>
                <canvas id="infinite-canvas" tabindex="0" aria-label="Infinite canvas workspace" aria-describedby="canvas-status" title="Use arrow keys to pan, plus/minus to zoom, and 0 to reset the view." data-ready="false" data-pan-x="0" data-pan-y="0" data-zoom="1"></canvas>
            </section>
        </main>
        "#
    .to_string()
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static APP_INITIALIZED: Cell<bool> = Cell::new(false);
}

#[cfg(target_arch = "wasm32")]
fn canvas_css_size(canvas: &HtmlCanvasElement) -> Result<(f64, f64), JsValue> {
    let browser_window = window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;
    let viewport_width = browser_window
        .inner_width()?
        .as_f64()
        .unwrap_or(FALLBACK_VIEWPORT_WIDTH);
    let viewport_height = browser_window
        .inner_height()?
        .as_f64()
        .unwrap_or(FALLBACK_VIEWPORT_HEIGHT);

    let width = match canvas.client_width() {
        0 => (viewport_width - CANVAS_HORIZONTAL_MARGIN).max(MIN_CANVAS_EDGE),
        value => f64::from(value),
    };
    let height = match canvas.client_height() {
        0 => (viewport_height - CANVAS_VERTICAL_MARGIN).max(MIN_CANVAS_EDGE),
        value => f64::from(value),
    };

    Ok((width, height))
}

#[cfg(target_arch = "wasm32")]
fn resize_canvas(
    canvas: &HtmlCanvasElement,
    ctx: &CanvasRenderingContext2d,
) -> Result<(), JsValue> {
    let browser_window = window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;
    let (width, height) = canvas_css_size(canvas)?;
    // Keep CSS size stable while allocating a denser backing store for Retina/HiDPI displays.
    let device_pixel_ratio = browser_window.device_pixel_ratio().max(1.0);

    canvas
        .style()
        .set_property("width", &format!("{}px", width.round()))?;
    canvas
        .style()
        .set_property("height", &format!("{}px", height.round()))?;
    canvas.set_width((width * device_pixel_ratio).round() as u32);
    canvas.set_height((height * device_pixel_ratio).round() as u32);

    ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
    ctx.scale(device_pixel_ratio, device_pixel_ratio)?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn log_js_error(prefix: &str, error: &JsValue) {
    let message = error.as_string().unwrap_or_else(|| format!("{error:?}"));
    web_sys::console::error_1(&JsValue::from_str(&format!("{prefix}: {message}")));
}

#[cfg(target_arch = "wasm32")]
fn render_canvas(
    ctx: &CanvasRenderingContext2d,
    canvas: &HtmlCanvasElement,
    status: &HtmlElement,
    state: &ViewportState,
) -> Result<(), JsValue> {
    let (width, height) = canvas_css_size(canvas)?;
    let zoom = state.zoom;
    let grid_spacing = (GRID_BASE_SPACING * zoom).clamp(GRID_MIN_SPACING, GRID_MAX_SPACING);
    let offset_x = state.pan_x.rem_euclid(grid_spacing);
    let offset_y = state.pan_y.rem_euclid(grid_spacing);

    ctx.set_fill_style_str("#f8fafc");
    ctx.fill_rect(0.0, 0.0, width, height);

    ctx.begin_path();
    ctx.set_stroke_style_str("#d7e3f4");
    ctx.set_line_width(1.0);

    let mut x = offset_x - grid_spacing;
    while x <= width + grid_spacing {
        ctx.move_to(x, 0.0);
        ctx.line_to(x, height);
        x += grid_spacing;
    }

    let mut y = offset_y - grid_spacing;
    while y <= height + grid_spacing {
        ctx.move_to(0.0, y);
        ctx.line_to(width, y);
        y += grid_spacing;
    }
    ctx.stroke();

    let center_x = (width / 2.0) + state.pan_x;
    let center_y = (height / 2.0) + state.pan_y;

    ctx.begin_path();
    ctx.set_stroke_style_str("#2563eb");
    ctx.set_line_width(2.0);
    ctx.move_to(center_x - (20.0 * zoom), center_y);
    ctx.line_to(center_x + (20.0 * zoom), center_y);
    ctx.move_to(center_x, center_y - (20.0 * zoom));
    ctx.line_to(center_x, center_y + (20.0 * zoom));
    ctx.stroke();

    let note_x = (width / 2.0) + state.pan_x + (110.0 * zoom);
    let note_y = (height / 2.0) + state.pan_y + (80.0 * zoom);
    let note_width = 230.0 * zoom;
    let note_height = 112.0 * zoom;

    ctx.set_fill_style_str("#fef3c7");
    ctx.fill_rect(note_x, note_y, note_width, note_height);
    ctx.set_stroke_style_str("#f59e0b");
    ctx.stroke_rect(note_x, note_y, note_width, note_height);

    ctx.set_fill_style_str("#111827");
    ctx.set_font(&format!("{}px sans-serif", (18.0 * zoom).clamp(12.0, 28.0)));
    ctx.fill_text(
        "Infinite canvas",
        note_x + (16.0 * zoom),
        note_y + (32.0 * zoom),
    )?;
    ctx.set_font(&format!("{}px sans-serif", (13.0 * zoom).clamp(10.0, 20.0)));
    ctx.fill_text(
        "Drag anywhere to keep exploring.",
        note_x + (16.0 * zoom),
        note_y + (58.0 * zoom),
    )?;
    ctx.fill_text(
        "The grid keeps moving forever.",
        note_x + (16.0 * zoom),
        note_y + (80.0 * zoom),
    )?;

    canvas.set_attribute("data-ready", "true")?;
    canvas.set_attribute("data-pan-x", &format!("{:.2}", state.pan_x))?;
    canvas.set_attribute("data-pan-y", &format!("{:.2}", state.pan_y))?;
    canvas.set_attribute("data-zoom", &format!("{:.2}", state.zoom))?;
    canvas.style().set_property(
        "cursor",
        if state.is_dragging {
            "grabbing"
        } else {
            "grab"
        },
    )?;

    status.set_text_content(Some(&format!(
        "Pan ({:.0}, {:.0}) · Zoom {:.2}× · {}",
        state.pan_x, state.pan_y, state.zoom, STATUS_HELP_TEXT
    )));

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn install_app(document: &Document) -> Result<(HtmlCanvasElement, HtmlElement), JsValue> {
    let body = document
        .body()
        .ok_or_else(|| JsValue::from_str("document has no body element"))?;
    body.set_inner_html(&app_markup());

    let canvas = document
        .get_element_by_id("infinite-canvas")
        .ok_or_else(|| JsValue::from_str("canvas element not found"))?
        .dyn_into::<HtmlCanvasElement>()?;
    let status = document
        .get_element_by_id("canvas-status")
        .ok_or_else(|| JsValue::from_str("status element not found"))?
        .dyn_into::<HtmlElement>()?;

    Ok((canvas, status))
}

#[cfg(target_arch = "wasm32")]
fn end_drag_if_needed(state: &Rc<RefCell<ViewportState>>, render: &Rc<dyn Fn()>) {
    if state.borrow().is_dragging {
        state.borrow_mut().end_drag();
        render();
    }
}

#[cfg(target_arch = "wasm32")]
fn start_impl() -> Result<(), JsValue> {
    let browser_window = window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;
    let document = browser_window
        .document()
        .ok_or_else(|| JsValue::from_str("could not access the browser document"))?;

    let (canvas, status) = install_app(&document)?;

    let context = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("could not access the canvas context"))?
        .dyn_into::<CanvasRenderingContext2d>()?;
    resize_canvas(&canvas, &context)?;

    let state = Rc::new(RefCell::new(ViewportState::default()));
    let is_rendering = Rc::new(Cell::new(false));
    let render: Rc<dyn Fn()> = Rc::new({
        let context = context.clone();
        let canvas = canvas.clone();
        let status = status.clone();
        let state = state.clone();
        let is_rendering = is_rendering.clone();
        move || {
            if is_rendering.replace(true) {
                return;
            }

            let snapshot = state.borrow().clone();
            if let Err(error) = render_canvas(&context, &canvas, &status, &snapshot) {
                log_js_error("render_canvas failed", &error);
            }

            is_rendering.set(false);
        }
    });
    render();

    let on_mouse_down = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let state = state.clone();
        let render = render.clone();
        move |event: MouseEvent| {
            if event.button() != 0 {
                return;
            }

            event.prevent_default();
            if let Err(error) = canvas.focus() {
                log_js_error("canvas focus failed", &error);
            }
            state
                .borrow_mut()
                .start_drag(event.client_x() as f64, event.client_y() as f64);
            render();
        }
    }));
    canvas.add_event_listener_with_callback("mousedown", on_mouse_down.as_ref().unchecked_ref())?;
    // `forget()` intentionally transfers listener ownership to JS for the app lifetime.
    on_mouse_down.forget();

    let on_mouse_move = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new({
        let state = state.clone();
        let render = render.clone();
        move |event: MouseEvent| {
            let did_move = {
                state
                    .borrow_mut()
                    .drag_to(event.client_x() as f64, event.client_y() as f64)
            };

            if did_move {
                render();
            }
        }
    }));
    browser_window
        .add_event_listener_with_callback("mousemove", on_mouse_move.as_ref().unchecked_ref())?;
    on_mouse_move.forget();

    let on_mouse_up = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new({
        let state = state.clone();
        let render = render.clone();
        move |_event: MouseEvent| end_drag_if_needed(&state, &render)
    }));
    browser_window
        .add_event_listener_with_callback("mouseup", on_mouse_up.as_ref().unchecked_ref())?;
    on_mouse_up.forget();

    let on_mouse_leave = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new({
        let state = state.clone();
        let render = render.clone();
        move |_event: MouseEvent| end_drag_if_needed(&state, &render)
    }));
    document
        .add_event_listener_with_callback("mouseleave", on_mouse_leave.as_ref().unchecked_ref())?;
    on_mouse_leave.forget();

    let on_blur = Closure::<dyn FnMut()>::wrap(Box::new({
        let state = state.clone();
        let render = render.clone();
        move || end_drag_if_needed(&state, &render)
    }));
    browser_window.add_event_listener_with_callback("blur", on_blur.as_ref().unchecked_ref())?;
    on_blur.forget();

    let on_wheel = Closure::<dyn FnMut(WheelEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let state = state.clone();
        let render = render.clone();
        move |event: WheelEvent| {
            event.prevent_default();
            let factor = if event.delta_y() < 0.0 {
                ZOOM_STEP_FACTOR
            } else {
                1.0 / ZOOM_STEP_FACTOR
            };
            let viewport_width = f64::from(canvas.client_width().max(1));
            let viewport_height = f64::from(canvas.client_height().max(1));

            state.borrow_mut().zoom_at(
                factor,
                event.offset_x() as f64,
                event.offset_y() as f64,
                viewport_width,
                viewport_height,
            );
            render();
        }
    }));
    canvas.add_event_listener_with_callback("wheel", on_wheel.as_ref().unchecked_ref())?;
    on_wheel.forget();

    let on_key_down = Closure::<dyn FnMut(KeyboardEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let state = state.clone();
        let render = render.clone();
        move |event: KeyboardEvent| {
            let viewport_width = f64::from(canvas.client_width().max(1));
            let viewport_height = f64::from(canvas.client_height().max(1));
            let mut viewport = state.borrow_mut();

            let handled = match event.key().as_str() {
                "ArrowLeft" => {
                    viewport.pan_by(-KEYBOARD_PAN_STEP, 0.0);
                    true
                }
                "ArrowRight" => {
                    viewport.pan_by(KEYBOARD_PAN_STEP, 0.0);
                    true
                }
                "ArrowUp" => {
                    viewport.pan_by(0.0, -KEYBOARD_PAN_STEP);
                    true
                }
                "ArrowDown" => {
                    viewport.pan_by(0.0, KEYBOARD_PAN_STEP);
                    true
                }
                "+" | "=" => {
                    viewport.zoom_at(
                        ZOOM_STEP_FACTOR,
                        viewport_width / 2.0,
                        viewport_height / 2.0,
                        viewport_width,
                        viewport_height,
                    );
                    true
                }
                "-" | "_" => {
                    viewport.zoom_at(
                        1.0 / ZOOM_STEP_FACTOR,
                        viewport_width / 2.0,
                        viewport_height / 2.0,
                        viewport_width,
                        viewport_height,
                    );
                    true
                }
                "0" | "Home" => {
                    viewport.reset();
                    true
                }
                _ => false,
            };

            drop(viewport);
            if handled {
                event.prevent_default();
                render();
            }
        }
    }));
    canvas.add_event_listener_with_callback("keydown", on_key_down.as_ref().unchecked_ref())?;
    on_key_down.forget();

    let on_resize = Closure::<dyn FnMut()>::wrap(Box::new({
        let canvas = canvas.clone();
        let context = context.clone();
        let render = render.clone();
        move || {
            if let Err(error) = resize_canvas(&canvas, &context) {
                log_js_error("resize_canvas failed", &error);
            }
            render();
        }
    }));
    browser_window
        .add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref())?;
    on_resize.forget();

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    if APP_INITIALIZED.with(|flag| flag.replace(true)) {
        return Ok(());
    }

    if let Err(error) = start_impl() {
        APP_INITIALIZED.with(|flag| flag.set(false));
        return Err(error);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ViewportState;

    #[test]
    fn default_viewport_state_is_centered() {
        let state = ViewportState::default();

        assert_eq!(state.pan_x, 0.0);
        assert_eq!(state.pan_y, 0.0);
        assert_eq!(state.zoom, 1.0);
        assert!(!state.is_dragging);
        assert_eq!(state.last_mouse_pos, None);
    }

    #[test]
    fn dragging_updates_pan_coordinates() {
        let mut state = ViewportState::default();

        state.start_drag(20.0, 40.0);
        assert!(state.drag_to(65.0, 95.0));
        assert_eq!(state.pan_x, 45.0);
        assert_eq!(state.pan_y, 55.0);

        assert!(state.drag_to(80.0, 125.0));
        assert_eq!(state.pan_x, 60.0);
        assert_eq!(state.pan_y, 85.0);
    }

    #[test]
    fn drag_stops_after_release_and_zoom_is_clamped() {
        let mut state = ViewportState::default();

        state.start_drag(0.0, 0.0);
        assert!(state.drag_to(12.0, -18.0));
        state.end_drag();
        assert!(!state.drag_to(30.0, 30.0));
        assert_eq!(state.pan_x, 12.0);
        assert_eq!(state.pan_y, -18.0);

        for _ in 0..12 {
            state.zoom_by(1.3);
        }
        assert_eq!(state.zoom, 2.5);

        for _ in 0..24 {
            state.zoom_by(0.5);
        }
        assert_eq!(state.zoom, 0.5);
    }

    #[test]
    fn panning_by_delta_moves_the_viewport() {
        let mut state = ViewportState::default();

        state.pan_by(24.0, -16.0);
        state.pan_by(-10.0, 6.0);

        assert_eq!(state.pan_x, 14.0);
        assert_eq!(state.pan_y, -10.0);

        state.reset();
        assert_eq!(state, ViewportState::default());
    }

    #[test]
    fn zooming_keeps_the_cursor_world_point_stable() {
        let mut state = ViewportState::default();
        let viewport_width = 800.0;
        let viewport_height = 600.0;
        let cursor_x = 620.0;
        let cursor_y = 420.0;

        let world_before =
            state.world_point_at(cursor_x, cursor_y, viewport_width, viewport_height);
        state.zoom_at(1.25, cursor_x, cursor_y, viewport_width, viewport_height);
        let world_after = state.world_point_at(cursor_x, cursor_y, viewport_width, viewport_height);

        assert!((world_before.0 - world_after.0).abs() < 1e-9);
        assert!((world_before.1 - world_after.1).abs() < 1e-9);
        assert!(state.zoom > 1.0);
    }
}
