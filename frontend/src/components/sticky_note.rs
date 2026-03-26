use crate::constants::*;
use crate::state::AppState;
use crate::styles::StickyNoteStyle;
use cocomiro_shared::{Position, StickyNote};
use wasm_bindgen::JsCast;
use web_sys::{HtmlTextAreaElement, InputEvent, KeyboardEvent, MouseEvent};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StickyNoteProps {
    pub note: StickyNote,
    pub app_state: UseReducerHandle<AppState>,
    pub is_editing: bool,
    pub editing_content: Option<String>,
    pub is_selected: bool,
    pub on_start_edit: Callback<String>,
    pub on_save_edit: Callback<MouseEvent>,
    pub on_cancel_edit: Callback<MouseEvent>,
    pub on_update_content: Callback<String>,
    pub on_select: Callback<String>,
}

#[function_component(StickyNoteComponent)]
pub fn sticky_note_component(props: &StickyNoteProps) -> Html {
    let styles = StickyNoteStyle::new();
    let note = &props.note;
    let view_state = &props.app_state.view;

    // Drag state
    let is_dragging = use_state(|| false);
    let drag_start_pos = use_state(|| (0.0, 0.0));
    let note_start_pos = use_state(|| (0.0, 0.0));
    let is_hovered = use_state(|| false);

    // Ref for the textarea
    let textarea_ref = use_node_ref();

    // Focus textarea when entering edit mode
    {
        let textarea_ref = textarea_ref.clone();
        let is_editing = props.is_editing;
        use_effect_with_deps(
            move |_| {
                if is_editing {
                    if let Some(textarea) = textarea_ref.cast::<HtmlTextAreaElement>() {
                        let _ = textarea.focus();
                    }
                }
            },
            is_editing,
        );
    }

    // Transform note position based on view state
    let screen_x = note.position.x * view_state.zoom + view_state.pan_x;
    let screen_y = note.position.y * view_state.zoom + view_state.pan_y;
    let screen_width = note.size.width * view_state.zoom;
    let screen_height = note.size.height * view_state.zoom;

    let transform_style = StickyNoteStyle::calculate_transform(
        screen_x,
        screen_y,
        screen_width,
        screen_height,
        view_state.zoom,
    );

    // Determine CSS classes
    let combined_classes = if props.is_editing {
        // When editing, show editing style (red border) plus selected if it was selected
        if props.is_selected {
            classes![
                styles.base.clone(),
                styles.selected.clone(),
                styles.editing.clone()
            ]
        } else {
            classes![styles.base.clone(), styles.editing.clone()]
        }
    } else if props.is_selected && *is_dragging {
        classes![
            styles.base.clone(),
            styles.selected.clone(),
            styles.dragging.clone()
        ]
    } else if props.is_selected {
        classes![styles.base.clone(), styles.selected.clone()]
    } else if *is_dragging {
        classes![styles.base.clone(), styles.dragging.clone()]
    } else {
        styles.base.clone()
    };

    if props.is_editing {
        // Edit mode - show textarea
        let on_input = {
            let on_update_content = props.on_update_content.clone();
            Callback::from(move |e: InputEvent| {
                if let Ok(target) = e.target().unwrap().dyn_into::<HtmlTextAreaElement>() {
                    let value = target.value();
                    on_update_content.emit(value);
                }
            })
        };

        let on_keydown = {
            let on_save_edit = props.on_save_edit.clone();
            let on_cancel_edit = props.on_cancel_edit.clone();
            Callback::from(move |e: KeyboardEvent| {
                if e.key() == "Enter" && !e.shift_key() {
                    e.prevent_default();
                    if let Ok(event) = web_sys::MouseEvent::new("click") {
                        on_save_edit.emit(event);
                    }
                } else if e.key() == "Escape" {
                    e.prevent_default();
                    if let Ok(event) = web_sys::MouseEvent::new("click") {
                        on_cancel_edit.emit(event);
                    }
                }
            })
        };

        html! {
            <textarea
                ref={textarea_ref}
                class={classes![styles.base, styles.editing]}
                style={transform_style}
                value={props.editing_content.clone().unwrap_or_default()}
                oninput={on_input}
                onkeydown={on_keydown}
                autofocus={true}
                data-testid="sticky-note-textarea"
            />
        }
    } else {
        // Display mode - show text
        let on_click = {
            let note_id = note.id.clone();
            let on_select = props.on_select.clone();
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                on_select.emit(note_id.clone());
            })
        };

        let on_dblclick = {
            let note_id = note.id.clone();
            let on_start_edit = props.on_start_edit.clone();
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                on_start_edit.emit(note_id.clone());
            })
        };

        let on_mousedown = {
            let is_dragging = is_dragging.clone();
            let drag_start_pos = drag_start_pos.clone();
            let note_start_pos = note_start_pos.clone();
            let note_pos = (note.position.x, note.position.y);
            Callback::from(move |e: MouseEvent| {
                e.prevent_default();
                is_dragging.set(true);
                drag_start_pos.set((e.client_x() as f64, e.client_y() as f64));
                note_start_pos.set(note_pos);
            })
        };

        let on_mousemove = {
            let is_dragging = is_dragging.clone();
            let drag_start_pos = drag_start_pos.clone();
            let note_start_pos = note_start_pos.clone();
            let app_state = props.app_state.clone();
            let note_id = note.id.clone();
            let zoom = view_state.zoom;
            Callback::from(move |e: MouseEvent| {
                if *is_dragging {
                    e.prevent_default();
                    let current_pos = (e.client_x() as f64, e.client_y() as f64);
                    let delta_x = current_pos.0 - drag_start_pos.0;
                    let delta_y = current_pos.1 - drag_start_pos.1;
                    let world_delta_x = delta_x / zoom;
                    let world_delta_y = delta_y / zoom;
                    let new_pos = Position {
                        x: note_start_pos.0 + world_delta_x,
                        y: note_start_pos.1 + world_delta_y,
                    };
                    app_state.dispatch(crate::state::AppAction::StickyNotes(
                        crate::state::StickyNotesAction::UpdatePosition(note_id.clone(), new_pos),
                    ));
                }
            })
        };

        let on_mouseup = {
            let is_dragging = is_dragging.clone();
            Callback::from(move |_| {
                is_dragging.set(false);
            })
        };

        let on_mouseenter = {
            let is_hovered = is_hovered.clone();
            Callback::from(move |_| {
                is_hovered.set(true);
            })
        };

        let on_mouseleave = {
            let is_hovered = is_hovered.clone();
            Callback::from(move |_| {
                is_hovered.set(false);
            })
        };

        let cursor_style = if *is_dragging {
            "cursor: grabbing;"
        } else if *is_hovered {
            "cursor: grab;"
        } else {
            "cursor: pointer;"
        };

        let final_style = format!("{}{}", transform_style, cursor_style);

        html! {
            <div class={combined_classes} style={final_style} onclick={on_click} ondblclick={on_dblclick} onmousedown={on_mousedown} onmousemove={on_mousemove} onmouseup={on_mouseup} onmouseenter={on_mouseenter} onmouseleave={on_mouseleave} data-testid="sticky-note">
                { for note.content.split('\n').map(|line| {
                    html! {
                        <div style={format!("margin-bottom: {}px;", LINE_MARGIN * view_state.zoom.max(0.5))}>
                            {line}
                        </div>
                    }
                })}
            </div>
        }
    }
}
