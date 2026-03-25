use crate::constants::*;
use stylist::css;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct ToolbarStyle {
    pub container: Classes,
    pub handle: Classes,
    pub button: Classes,
    pub zoom_button: Classes,
    pub create_button: Classes,
}

impl ToolbarStyle {
    pub fn new() -> Self {
        Self {
            container: css!(
                r#"
                    position: absolute;
                    z-index: 10;
                    display: flex;
                    flex-direction: column;
                    gap: ${gap};
                    background: rgba(255, 255, 255, 0.9);
                    border-radius: 8px;
                    padding: ${padding};
                    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
                    backdrop-filter: blur(10px);
                    user-select: none;
                    touch-action: none;
                "#,
                gap = TOOLBAR_GAP,
                padding = TOOLBAR_PADDING,
            )
            .into(),
            handle: css!(
                r#"
                    width: 100%;
                    height: ${height};
                    background: linear-gradient(90deg, #ccc 25%, transparent 25%, transparent 75%, #ccc 75%);
                    background-size: 4px 4px;
                    border-radius: 4px 4px 0 0;
                    margin: ${margin};
                    cursor: grab;
                "#,
                height = TOOLBAR_HANDLE_HEIGHT,
                margin = TOOLBAR_HANDLE_MARGIN,
            )
            .into(),
            button: css!(
                r#"
                    width: ${size};
                    height: ${size};
                    border: ${border};
                    border-radius: 4px;
                    background: ${bg};
                    cursor: pointer;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 16px;
                    font-weight: bold;
                "#,
                size = BUTTON_SIZE,
                border = BUTTON_BORDER,
                bg = BUTTON_BG,
            )
            .into(),
            zoom_button: css!("").into(), // Inherits from button
            create_button: css!(
                r#"
                    background: ${bg};
                "#,
                bg = STICKY_BUTTON_BG,
            )
            .into(),
        }
    }

    pub fn calculate_position(x: f64, y: f64, is_dragging: bool) -> String {
        let cursor = if is_dragging { "grabbing" } else { "grab" };
        format!("left: {}px; top: {}px; cursor: {};", x, y, cursor)
    }
}
