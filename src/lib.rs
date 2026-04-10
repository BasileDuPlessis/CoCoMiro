#[cfg(target_arch = "wasm32")]
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, window};

pub mod app;
pub mod canvas;
pub mod events;
pub mod sticky_notes;
pub mod toolbar;
pub mod viewport;

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct AppState {
    pub viewport: viewport::ViewportState,
    pub sticky_notes: sticky_notes::StickyNotesState,
}

#[cfg(any(test, target_arch = "wasm32"))]
impl Default for AppState {
    fn default() -> Self {
        Self {
            viewport: viewport::ViewportState::default(),
            sticky_notes: sticky_notes::StickyNotesState::default(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static APP_INITIALIZED: Cell<bool> = Cell::new(false);
}

#[cfg(target_arch = "wasm32")]
pub fn log_js_error(prefix: &str, error: &JsValue) {
    let message = error.as_string().unwrap_or_else(|| format!("{error:?}"));
    web_sys::console::error_1(&JsValue::from_str(&format!("{prefix}: {message}")));
}

#[cfg(target_arch = "wasm32")]
pub fn log_info(message: &str) {
    web_sys::console::log_1(&JsValue::from_str(message));
}

#[cfg(target_arch = "wasm32")]
pub fn log_warn(message: &str) {
    web_sys::console::warn_1(&JsValue::from_str(message));
}

#[cfg(target_arch = "wasm32")]
fn start_impl() -> Result<(), JsValue> {
    let browser_window = window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;
    let document = browser_window
        .document()
        .ok_or_else(|| JsValue::from_str("could not access the browser document"))?;

    let (workspace, canvas, status, toolbar) = app::install_app(&document)?;

    let context = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("could not access the canvas context"))?
        .dyn_into::<CanvasRenderingContext2d>()?;
    canvas::resize_canvas(&canvas, &context)?;

    let state = Rc::new(RefCell::new(AppState::default()));
    let toolbar_state = Rc::new(RefCell::new(toolbar::FloatingToolbarState::default()));
    let is_rendering = Rc::new(Cell::new(false));
    let render: Rc<dyn Fn()> = Rc::new({
        let context = context.clone();
        let canvas = canvas.clone();
        let status = status.clone();
        let state = state.clone();
        let is_rendering = is_rendering.clone();
        move || {
            if is_rendering.replace(true) {
                log_info("Render skipped: already rendering");
                return;
            }

            let snapshot = state.borrow().clone();
            if let Err(error) = canvas::render_canvas(&context, &canvas, &status, &snapshot) {
                log_js_error("render_canvas failed", &error);
            }
            is_rendering.set(false);
        }
    });
    let position_toolbar: Rc<dyn Fn()> = Rc::new({
        let workspace = workspace.clone();
        let toolbar = toolbar.clone();
        let toolbar_state = toolbar_state.clone();
        move || {
            if let Err(error) =
                canvas::sync_toolbar_position(&toolbar, &workspace, &mut toolbar_state.borrow_mut())
            {
                log_js_error("sync_toolbar_position failed", &error);
            }
        }
    });
    render();
    position_toolbar();

    events::setup_event_listeners(
        &canvas,
        &workspace,
        &toolbar,
        &state,
        &toolbar_state,
        &render,
        &position_toolbar,
    )?;

    let on_resize = Closure::<dyn FnMut()>::wrap(Box::new({
        let canvas = canvas.clone();
        let context = context.clone();
        let render = render.clone();
        let position_toolbar = position_toolbar.clone();
        move || {
            if let Err(error) = canvas::resize_canvas(&canvas, &context) {
                log_js_error("resize_canvas failed", &error);
            }
            render();
            position_toolbar();
        }
    }));
    browser_window
        .add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref())?;
    on_resize.forget();

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
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
