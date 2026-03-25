use stylist::css;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct CanvasStyle {
    pub container: Classes,
    pub canvas: Classes,
    pub overlay: Classes,
}

impl CanvasStyle {
    pub fn new() -> Self {
        Self {
            container: css!(
                r#"
                    position: relative;
                    width: 100vw;
                    height: 100vh;
                    overflow: hidden;
                "#
            )
            .into(),
            canvas: css!(
                r#"
                    cursor: grab;
                    display: block;
                "#
            )
            .into(),
            overlay: css!(
                r#"
                    position: absolute;
                    top: 0;
                    left: 0;
                    width: 100%;
                    height: 100%;
                    z-index: 10;
                    cursor: default;
                "#
            )
            .into(),
        }
    }
}
