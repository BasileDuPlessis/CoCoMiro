#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
#[cfg(target_arch = "wasm32")]
use web_sys::{
    CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement, MouseEvent, WheelEvent,
    window,
};

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
#[derive(Debug, Clone, PartialEq)]
struct ViewportState {
    pan_x: f64,
    pan_y: f64,
    zoom: f64,
    is_dragging: bool,
    last_mouse_pos: Option<(f64, f64)>,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
            is_dragging: false,
            last_mouse_pos: None,
        }
    }
}

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
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
            self.pan_x += x - last_x;
            self.pan_y += y - last_y;
            self.last_mouse_pos = Some((x, y));
            return true;
        }

        self.last_mouse_pos = Some((x, y));
        false
    }

    fn end_drag(&mut self) {
        self.is_dragging = false;
        self.last_mouse_pos = None;
    }

    fn zoom_by(&mut self, factor: f64) {
        self.zoom = (self.zoom * factor).clamp(0.5, 2.5);
    }
}

#[cfg(target_arch = "wasm32")]
fn app_markup() -> String {
    r#"
        <main class="app-shell">
            <section class="canvas-panel">
                <p id="canvas-status" class="canvas-status">Pan (0, 0) · Zoom 1.00× · Hold the mouse button and drag.</p>
                <canvas id="infinite-canvas" tabindex="0" data-pan-x="0" data-pan-y="0" data-zoom="1"></canvas>
            </section>
        </main>
        "#
    .to_string()
}

#[cfg(target_arch = "wasm32")]
fn resize_canvas(canvas: &HtmlCanvasElement) -> Result<(), JsValue> {
    let browser_window = window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;
    let viewport_width = browser_window.inner_width()?.as_f64().unwrap_or(1280.0);
    let viewport_height = browser_window.inner_height()?.as_f64().unwrap_or(840.0);

    let width = match canvas.client_width() {
        0 => (viewport_width - 32.0).max(320.0),
        value => f64::from(value),
    };
    let height = match canvas.client_height() {
        0 => (viewport_height - 96.0).max(320.0),
        value => f64::from(value),
    };

    canvas.set_width(width.round() as u32);
    canvas.set_height(height.round() as u32);
    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[allow(deprecated)]
fn render_canvas(
    ctx: &CanvasRenderingContext2d,
    canvas: &HtmlCanvasElement,
    status: &HtmlElement,
    state: &ViewportState,
) -> Result<(), JsValue> {
    let width = canvas.width() as f64;
    let height = canvas.height() as f64;
    let zoom = state.zoom;
    let grid_spacing = (48.0 * zoom).clamp(24.0, 120.0);
    let offset_x = state.pan_x.rem_euclid(grid_spacing);
    let offset_y = state.pan_y.rem_euclid(grid_spacing);

    ctx.set_fill_style(&JsValue::from_str("#f8fafc"));
    ctx.fill_rect(0.0, 0.0, width, height);

    ctx.begin_path();
    ctx.set_stroke_style(&JsValue::from_str("#d7e3f4"));
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
    ctx.set_stroke_style(&JsValue::from_str("#2563eb"));
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

    ctx.set_fill_style(&JsValue::from_str("#fef3c7"));
    ctx.fill_rect(note_x, note_y, note_width, note_height);
    ctx.set_stroke_style(&JsValue::from_str("#f59e0b"));
    ctx.stroke_rect(note_x, note_y, note_width, note_height);

    ctx.set_fill_style(&JsValue::from_str("#111827"));
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
        "Pan ({:.0}, {:.0}) · Zoom {:.2}× · Hold the mouse button and drag.",
        state.pan_x, state.pan_y, state.zoom
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
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let document = window()
        .and_then(|win| win.document())
        .ok_or_else(|| JsValue::from_str("could not access the browser document"))?;

    let (canvas, status) = install_app(&document)?;
    resize_canvas(&canvas)?;

    let context = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("could not access the canvas context"))?
        .dyn_into::<CanvasRenderingContext2d>()?;

    let state = Rc::new(RefCell::new(ViewportState::default()));
    let render: Rc<dyn Fn()> = Rc::new({
        let context = context.clone();
        let canvas = canvas.clone();
        let status = status.clone();
        let state = state.clone();
        move || {
            let snapshot = state.borrow().clone();
            let _ = render_canvas(&context, &canvas, &status, &snapshot);
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
            let _ = canvas.focus();
            state
                .borrow_mut()
                .start_drag(event.client_x() as f64, event.client_y() as f64);
            render();
        }
    }));
    canvas.add_event_listener_with_callback("mousedown", on_mouse_down.as_ref().unchecked_ref())?;
    on_mouse_down.forget();

    let browser_window = window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;

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
        move |_event: MouseEvent| {
            if state.borrow().is_dragging {
                state.borrow_mut().end_drag();
                render();
            }
        }
    }));
    browser_window
        .add_event_listener_with_callback("mouseup", on_mouse_up.as_ref().unchecked_ref())?;
    on_mouse_up.forget();

    let on_wheel = Closure::<dyn FnMut(WheelEvent)>::wrap(Box::new({
        let state = state.clone();
        let render = render.clone();
        move |event: WheelEvent| {
            event.prevent_default();
            let factor = if event.delta_y() < 0.0 {
                1.1
            } else {
                1.0 / 1.1
            };
            state.borrow_mut().zoom_by(factor);
            render();
        }
    }));
    canvas.add_event_listener_with_callback("wheel", on_wheel.as_ref().unchecked_ref())?;
    on_wheel.forget();

    let on_resize = Closure::<dyn FnMut()>::wrap(Box::new({
        let canvas = canvas.clone();
        let render = render.clone();
        move || {
            let _ = resize_canvas(&canvas);
            render();
        }
    }));
    browser_window
        .add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref())?;
    on_resize.forget();

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
}
