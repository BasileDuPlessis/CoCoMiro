use crate::constants::*;
use crate::state::AppState;
use hello_world_shared::StickyNote;
use wasm_bindgen::JsCast;
use web_sys::{HtmlTextAreaElement, InputEvent, KeyboardEvent, MouseEvent};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StickyNoteProps {
    pub note: StickyNote,
    pub app_state: UseReducerHandle<AppState>,
    pub is_editing: bool,
    pub editing_content: Option<String>,
    pub on_start_edit: Callback<String>,
    pub on_save_edit: Callback<MouseEvent>,
    pub on_cancel_edit: Callback<MouseEvent>,
    pub on_update_content: Callback<String>,
}

#[function_component(StickyNoteComponent)]
pub fn sticky_note_component(props: &StickyNoteProps) -> Html {
    let note = &props.note;
    let view_state = &props.app_state.view;

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

    let style = format!(
        "position: absolute; left: {}px; top: {}px; width: {}px; height: {}px; background: {}; border: 2px solid {}; padding: 8px; box-sizing: border-box; font-family: Arial, sans-serif; font-size: {}px; cursor: pointer; user-select: none; z-index: 5;",
        screen_x,
        screen_y,
        screen_width,
        screen_height,
        STICKY_NOTE_BG,
        STICKY_NOTE_BORDER,
        FONT_SIZE_BASE * view_state.zoom.max(0.5)
    );

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
                style={format!("{} resize: none; border: none; outline: none; background: {};", style, STICKY_NOTE_BG)}
                value={props.editing_content.clone().unwrap_or_default()}
                oninput={on_input}
                onkeydown={on_keydown}
                autofocus={true}
            />
        }
    } else {
        // Display mode - show text
        let on_click = {
            let note_id = note.id.clone();
            let on_start_edit = props.on_start_edit.clone();
            Callback::from(move |_| {
                on_start_edit.emit(note_id.clone());
            })
        };

        html! {
            <div {style} onclick={on_click}>
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
