use crate::constants::*;
use crate::state::{AppAction, AppState, ToolbarAction};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, MouseEvent, PointerEvent};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FloatingToolbarProps {
    pub app_state: UseReducerHandle<AppState>,
    pub on_zoom_in: Callback<MouseEvent>,
    pub on_zoom_out: Callback<MouseEvent>,
    pub on_create_sticky_note: Callback<MouseEvent>,
}

#[function_component(FloatingToolbar)]
pub fn floating_toolbar(props: &FloatingToolbarProps) -> Html {
    let app_state = &props.app_state;

    let on_pointer_down = {
        let app_state = app_state.clone();
        Callback::from(move |e: PointerEvent| {
            e.prevent_default();
            e.stop_propagation();

            // Capture the pointer to receive events even when outside the element
            if let Some(target) = e.target() {
                if let Ok(html_element) = target.dyn_into::<HtmlElement>() {
                    let _ = html_element.set_pointer_capture(e.pointer_id());
                }
            }

            let offset_x = e.client_x() as f64 - app_state.toolbar.x;
            let offset_y = e.client_y() as f64 - app_state.toolbar.y;

            app_state.dispatch(AppAction::Toolbar(ToolbarAction::StartDrag(
                offset_x, offset_y,
            )));
        })
    };

    let on_pointer_move = {
        let app_state = app_state.clone();
        Callback::from(move |e: PointerEvent| {
            if app_state.toolbar.is_dragging {
                e.prevent_default();
                app_state.dispatch(AppAction::Toolbar(ToolbarAction::UpdateDrag(
                    e.client_x() as f64,
                    e.client_y() as f64,
                )));
            }
        })
    };

    let on_pointer_up = {
        let app_state = app_state.clone();
        Callback::from(move |e: PointerEvent| {
            e.prevent_default();
            app_state.dispatch(AppAction::Toolbar(ToolbarAction::EndDrag));
        })
    };

    let style = format!(
        "position: absolute; left: {}px; top: {}px; z-index: 10; display: flex; flex-direction: column; gap: {}; background: rgba(255, 255, 255, 0.9); border-radius: 8px; padding: {}; box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15); backdrop-filter: blur(10px); cursor: {}; user-select: none; touch-action: none;",
        app_state.toolbar.x,
        app_state.toolbar.y,
        TOOLBAR_GAP,
        TOOLBAR_PADDING,
        if app_state.toolbar.is_dragging { "grabbing" } else { "grab" }
    );

    html! {
        <div
            {style}
            onpointerdown={on_pointer_down}
            onpointermove={on_pointer_move}
            onpointerup={on_pointer_up}
        >
            <div style={format!("
                width: 100%;
                height: {};
                background: linear-gradient(90deg, #ccc 25%, transparent 25%, transparent 75%, #ccc 75%);
                background-size: 4px 4px;
                border-radius: 4px 4px 0 0;
                margin: {} {};
                cursor: grab;
            ", TOOLBAR_HANDLE_HEIGHT, TOOLBAR_HANDLE_MARGIN, TOOLBAR_GAP)} title="Drag to move toolbar"></div>
            <button
                onclick={&props.on_zoom_in}
                style={format!("
                    width: {};
                    height: {};
                    border: {};
                    border-radius: 4px;
                    background: {};
                    cursor: pointer;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 16px;
                    font-weight: bold;
                ", BUTTON_SIZE, BUTTON_SIZE, BUTTON_BORDER, BUTTON_BG)}
                title="Zoom In"
            >
                {"+"}
            </button>
            <button
                onclick={&props.on_zoom_out}
                style={format!("
                    width: {};
                    height: {};
                    border: {};
                    border-radius: 4px;
                    background: {};
                    cursor: pointer;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 16px;
                    font-weight: bold;
                ", BUTTON_SIZE, BUTTON_SIZE, BUTTON_BORDER, BUTTON_BG)}
                title="Zoom Out"
            >
                {"-"}
            </button>
            <button
                onclick={&props.on_create_sticky_note}
                style={format!("
                    width: {};
                    height: {};
                    border: {};
                    border-radius: 4px;
                    background: {};
                    cursor: pointer;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 16px;
                    font-weight: bold;
                ", BUTTON_SIZE, BUTTON_SIZE, BUTTON_BORDER, STICKY_BUTTON_BG)}
                title="Create Sticky Note"
            >
                {"📝"}
            </button>
        </div>
    }
}
