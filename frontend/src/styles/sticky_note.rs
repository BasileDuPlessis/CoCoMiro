use crate::constants::*;
use stylist::css;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct StickyNoteStyle {
    pub base: Classes,
    pub selected: Classes,
    pub dragging: Classes,
    pub editing: Classes,
}

impl StickyNoteStyle {
    pub fn new() -> Self {
        Self {
            base: css!(
                r#"
                    position: absolute;
                    background: ${bg};
                    border: 2px solid ${border};
                    padding: 8px;
                    box-sizing: border-box;
                    font-family: Arial, sans-serif;
                    cursor: pointer;
                    user-select: none;
                    z-index: 5;
                "#,
                bg = STICKY_NOTE_BG,
                border = STICKY_NOTE_BORDER,
            )
            .into(),
            selected: css!(
                r#"
                    border-color: ${selected_border};
                    box-shadow: 0 0 8px rgba(0, 123, 255, 0.5);
                "#,
                selected_border = STICKY_NOTE_SELECTED_BORDER,
            )
            .into(),
            dragging: css!(
                r#"
                    cursor: grabbing;
                    box-shadow: ${shadow};
                    opacity: ${opacity};
                "#,
                shadow = STICKY_NOTE_DRAG_SHADOW,
                opacity = STICKY_NOTE_DRAG_OPACITY,
            )
            .into(),
            editing: css!(
                r#"
                    resize: none;
                    border: none;
                    outline: none;
                    background: ${bg};
                "#,
                bg = STICKY_NOTE_BG,
            )
            .into(),
        }
    }

    pub fn calculate_transform(x: f64, y: f64, width: f64, height: f64, zoom: f64) -> String {
        format!(
            "left: {}px; top: {}px; width: {}px; height: {}px; font-size: {}px;",
            x,
            y,
            width,
            height,
            FONT_SIZE_BASE * zoom.max(0.5)
        )
    }
}
