#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
#[cfg(target_arch = "wasm32")]
use web_sys::{HtmlCanvasElement, HtmlElement, KeyboardEvent, MouseEvent, WheelEvent, window};

#[cfg(target_arch = "wasm32")]
const ZOOM_STEP_FACTOR: f64 = 1.1;
#[cfg(target_arch = "wasm32")]
const KEYBOARD_PAN_STEP: f64 = 40.0;

#[cfg(target_arch = "wasm32")]
pub fn end_drag_if_needed(state: &Rc<RefCell<crate::AppState>>, render: &Rc<dyn Fn()>) {
    let mut should_render = false;
    {
        let mut app_state = state.borrow_mut();
        if app_state.viewport.is_dragging {
            app_state.viewport.end_drag();
            should_render = true;
        }
        if app_state.sticky_notes.is_dragging {
            app_state.sticky_notes.end_drag();
            should_render = true;
        }
    }
    if should_render {
        render();
    }
}

#[cfg(target_arch = "wasm32")]
pub fn end_toolbar_drag_if_needed(
    state: &Rc<RefCell<crate::toolbar::FloatingToolbarState>>,
    position_toolbar: &Rc<dyn Fn()>,
) {
    if state.borrow().is_dragging {
        state.borrow_mut().end_drag();
        position_toolbar();
    }
}

#[cfg(target_arch = "wasm32")]
pub fn setup_event_listeners(
    canvas: &HtmlCanvasElement,
    _workspace: &HtmlElement,
    toolbar: &HtmlElement,
    state: &Rc<RefCell<crate::AppState>>,
    toolbar_state: &Rc<RefCell<crate::toolbar::FloatingToolbarState>>,
    render: &Rc<dyn Fn()>,
    position_toolbar: &Rc<dyn Fn()>,
) -> Result<(), JsValue> {
    let browser_window = window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;
    let document = browser_window
        .document()
        .ok_or_else(|| JsValue::from_str("could not access the browser document"))?;

    // Mouse down on canvas
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
                crate::log_js_error("canvas focus failed", &error);
            }

            let mouse_x = event.client_x() as f64;
            let mouse_y = event.client_y() as f64;

            // Check for sticky note first
            let note_hit = {
                let viewport_width = f64::from(canvas.client_width().max(1));
                let viewport_height = f64::from(canvas.client_height().max(1));
                let world_pos = state.borrow().viewport.world_point_at(
                    mouse_x,
                    mouse_y,
                    viewport_width,
                    viewport_height,
                );
                state
                    .borrow()
                    .sticky_notes
                    .find_note_at(world_pos.0, world_pos.1)
            };

            if let Some(note_id) = note_hit {
                // Start dragging the sticky note
                let world_pos = {
                    let viewport_width = f64::from(canvas.client_width().max(1));
                    let viewport_height = f64::from(canvas.client_height().max(1));
                    state.borrow().viewport.world_point_at(
                        mouse_x,
                        mouse_y,
                        viewport_width,
                        viewport_height,
                    )
                };
                state
                    .borrow_mut()
                    .sticky_notes
                    .start_drag(note_id, world_pos.0, world_pos.1);
                crate::log_info(&format!("Sticky note {} drag started", note_id));
                render();
                return;
            }

            // If no sticky note hit, start canvas drag
            state.borrow_mut().viewport.start_drag(mouse_x, mouse_y);
            crate::log_info(&format!(
                "Canvas drag started at ({}, {})",
                mouse_x, mouse_y
            ));
            render();
        }
    }));
    canvas.add_event_listener_with_callback("mousedown", on_mouse_down.as_ref().unchecked_ref())?;
    on_mouse_down.forget();

    // Mouse down on toolbar handle
    let on_toolbar_mouse_down = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let toolbar_state = toolbar_state.clone();
        let position_toolbar = position_toolbar.clone();
        move |event: MouseEvent| {
            if event.button() != 0 {
                return;
            }

            let Some(target) = event.target() else {
                return;
            };
            let Ok(target_element) = target.dyn_into::<HtmlElement>() else {
                return;
            };
            if target_element.id() != "floating-toolbar-handle" {
                return;
            }

            event.prevent_default();
            event.stop_propagation();
            if let Err(error) = canvas.focus() {
                crate::log_js_error("canvas focus failed", &error);
            }
            toolbar_state
                .borrow_mut()
                .start_drag(event.client_x() as f64, event.client_y() as f64);
            position_toolbar();
        }
    }));
    toolbar.add_event_listener_with_callback(
        "mousedown",
        on_toolbar_mouse_down.as_ref().unchecked_ref(),
    )?;
    on_toolbar_mouse_down.forget();

    // Click on add note button
    let on_add_note_click = Closure::<dyn FnMut()>::wrap(Box::new({
        let canvas = canvas.clone();
        let state = state.clone();
        let render = render.clone();
        move || {
            let viewport_width = f64::from(canvas.client_width().max(1));
            let viewport_height = f64::from(canvas.client_height().max(1));

            let viewport = state.borrow().viewport.clone();
            state.borrow_mut().sticky_notes.add_note_at_viewport_center(
                viewport_width,
                viewport_height,
                &viewport,
            );

            render();
            crate::log_info("Added new sticky note");
        }
    }));
    let add_note_button = document
        .get_element_by_id("add-note-button")
        .ok_or_else(|| JsValue::from_str("add note button element not found"))?
        .dyn_into::<web_sys::HtmlElement>()?;
    add_note_button
        .add_event_listener_with_callback("click", on_add_note_click.as_ref().unchecked_ref())?;
    on_add_note_click.forget();

    // Mouse move
    let on_mouse_move = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let state = state.clone();
        let render = render.clone();
        let toolbar_state = toolbar_state.clone();
        let position_toolbar = position_toolbar.clone();
        move |event: MouseEvent| {
            let mouse_x = event.client_x() as f64;
            let mouse_y = event.client_y() as f64;

            // Update mouse position in app state
            state.borrow_mut().mouse_x = mouse_x;
            state.borrow_mut().mouse_y = mouse_y;

            // Handle toolbar dragging first
            let did_toolbar_move = toolbar_state.borrow_mut().drag_to(mouse_x, mouse_y);
            if did_toolbar_move {
                position_toolbar();
                return;
            }

            // Handle sticky note dragging
            let did_note_move = {
                let viewport_width = f64::from(canvas.client_width().max(1));
                let viewport_height = f64::from(canvas.client_height().max(1));
                let world_pos = state.borrow().viewport.world_point_at(
                    mouse_x,
                    mouse_y,
                    viewport_width,
                    viewport_height,
                );
                let sticky_notes = &mut state.borrow_mut().sticky_notes;
                if sticky_notes.is_dragging {
                    sticky_notes.drag_to(world_pos.0, world_pos.1);
                    true
                } else {
                    false
                }
            };

            if did_note_move {
                render();
                return;
            }

            // Handle canvas dragging
            let did_move = { state.borrow_mut().viewport.drag_to(mouse_x, mouse_y) };

            if did_move {
                render();
            }
        }
    }));
    browser_window
        .add_event_listener_with_callback("mousemove", on_mouse_move.as_ref().unchecked_ref())?;
    on_mouse_move.forget();

    // Mouse up
    let on_mouse_up = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new({
        let state = state.clone();
        let render = render.clone();
        let toolbar_state = toolbar_state.clone();
        let position_toolbar = position_toolbar.clone();
        move |_event: MouseEvent| {
            let was_dragging = state.borrow().viewport.is_dragging;
            let toolbar_was_dragging = toolbar_state.borrow().is_dragging;
            let sticky_note_was_dragging = state.borrow().sticky_notes.is_dragging;
            end_drag_if_needed(&state, &render);
            end_toolbar_drag_if_needed(&toolbar_state, &position_toolbar);
            if was_dragging {
                crate::log_info("Canvas drag ended");
            }
            if toolbar_was_dragging {
                crate::log_info("Toolbar drag ended");
            }
            if sticky_note_was_dragging {
                crate::log_info("Sticky note drag ended");
            }
        }
    }));
    browser_window
        .add_event_listener_with_callback("mouseup", on_mouse_up.as_ref().unchecked_ref())?;
    on_mouse_up.forget();

    // Mouse leave document
    let on_mouse_leave = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new({
        let state = state.clone();
        let render = render.clone();
        let toolbar_state = toolbar_state.clone();
        let position_toolbar = position_toolbar.clone();
        move |_event: MouseEvent| {
            end_drag_if_needed(&state, &render);
            end_toolbar_drag_if_needed(&toolbar_state, &position_toolbar);
        }
    }));
    document
        .add_event_listener_with_callback("mouseleave", on_mouse_leave.as_ref().unchecked_ref())?;
    on_mouse_leave.forget();

    // Blur window
    let on_blur = Closure::<dyn FnMut()>::wrap(Box::new({
        let state = state.clone();
        let render = render.clone();
        let toolbar_state = toolbar_state.clone();
        let position_toolbar = position_toolbar.clone();
        move || {
            end_drag_if_needed(&state, &render);
            end_toolbar_drag_if_needed(&toolbar_state, &position_toolbar);
        }
    }));
    browser_window.add_event_listener_with_callback("blur", on_blur.as_ref().unchecked_ref())?;
    on_blur.forget();

    // Wheel
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

            let old_zoom = state.borrow().viewport.zoom;
            state.borrow_mut().viewport.zoom_at(
                factor,
                event.offset_x() as f64,
                event.offset_y() as f64,
                viewport_width,
                viewport_height,
            );
            let new_zoom = state.borrow().viewport.zoom;
            crate::log_info(&format!(
                "Zoom changed from {:.2} to {:.2} at ({}, {})",
                old_zoom,
                new_zoom,
                event.offset_x(),
                event.offset_y()
            ));
            render();
        }
    }));
    canvas.add_event_listener_with_callback("wheel", on_wheel.as_ref().unchecked_ref())?;
    on_wheel.forget();

    // Key down
    let on_key_down = Closure::<dyn FnMut(KeyboardEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let state = state.clone();
        let render = render.clone();
        move |event: KeyboardEvent| {
            let viewport_width = f64::from(canvas.client_width().max(1));
            let viewport_height = f64::from(canvas.client_height().max(1));
            let viewport = &mut state.borrow_mut().viewport;

            let handled = match event.key().as_str() {
                "ArrowLeft" => {
                    viewport.pan_by(-KEYBOARD_PAN_STEP, 0.0);
                    crate::log_info(&format!("Panned left by {}", KEYBOARD_PAN_STEP));
                    true
                }
                "ArrowRight" => {
                    viewport.pan_by(KEYBOARD_PAN_STEP, 0.0);
                    crate::log_info(&format!("Panned right by {}", KEYBOARD_PAN_STEP));
                    true
                }
                "ArrowUp" => {
                    viewport.pan_by(0.0, -KEYBOARD_PAN_STEP);
                    crate::log_info(&format!("Panned up by {}", KEYBOARD_PAN_STEP));
                    true
                }
                "ArrowDown" => {
                    viewport.pan_by(0.0, KEYBOARD_PAN_STEP);
                    crate::log_info(&format!("Panned down by {}", KEYBOARD_PAN_STEP));
                    true
                }
                "+" | "=" => {
                    let old_zoom = viewport.zoom;
                    viewport.zoom_at(
                        ZOOM_STEP_FACTOR,
                        viewport_width / 2.0,
                        viewport_height / 2.0,
                        viewport_width,
                        viewport_height,
                    );
                    crate::log_info(&format!(
                        "Zoomed in from {:.2} to {:.2}",
                        old_zoom, viewport.zoom
                    ));
                    true
                }
                "-" | "_" => {
                    let old_zoom = viewport.zoom;
                    viewport.zoom_at(
                        1.0 / ZOOM_STEP_FACTOR,
                        viewport_width / 2.0,
                        viewport_height / 2.0,
                        viewport_width,
                        viewport_height,
                    );
                    crate::log_info(&format!(
                        "Zoomed out from {:.2} to {:.2}",
                        old_zoom, viewport.zoom
                    ));
                    true
                }
                "0" | "Home" => {
                    viewport.reset();
                    crate::log_info("Viewport reset to default");
                    true
                }
                "Delete" | "Backspace" => {
                    if state.borrow().sticky_notes.selected_note_id.is_some() {
                        state.borrow_mut().sticky_notes.delete_selected();
                        crate::log_info("Deleted selected sticky note");
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if handled {
                event.prevent_default();
                render();
            }
        }
    }));
    canvas.add_event_listener_with_callback("keydown", on_key_down.as_ref().unchecked_ref())?;
    on_key_down.forget();

    Ok(())
}
