use crate::state::{AppAction, AppState, ToolbarAction};
use crate::styles::ToolbarStyle;
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
    let styles = ToolbarStyle::new();
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

    let position_style = ToolbarStyle::calculate_position(
        app_state.toolbar.x,
        app_state.toolbar.y,
        app_state.toolbar.is_dragging,
    );

    html! {
        <div
            class={styles.container}
            style={position_style}
            onpointerdown={on_pointer_down}
            onpointermove={on_pointer_move}
            onpointerup={on_pointer_up}
            data-testid="floating-toolbar"
        >
            <div class={styles.handle} title="Drag to move toolbar"></div>
            <button
                class={styles.button.clone()}
                onclick={&props.on_zoom_in}
                title="Zoom In"
            >
                {"+"}
            </button>
            <button
                class={styles.button.clone()}
                onclick={&props.on_zoom_out}
                title="Zoom Out"
            >
                {"-"}
            </button>
            <button
                class={classes![styles.button, styles.create_button]}
                onclick={&props.on_create_sticky_note}
                title="Create Sticky Note"
            >
                {"📝"}
            </button>
        </div>
    }
}
