#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use web_sys::{Document, HtmlCanvasElement, HtmlElement};

#[cfg(target_arch = "wasm32")]
pub fn app_markup() -> String {
    r#"
        <main class="app-shell">
            <section class="canvas-panel">
                <p id="canvas-status" class="canvas-status" role="status" aria-live="polite">Pan (0, 0) · Zoom 1.00× · Drag to pan, scroll to zoom, or use the arrow keys and +/-.</p>
                <div id="canvas-workspace" class="canvas-workspace">
                    <canvas id="infinite-canvas" tabindex="0" aria-label="Infinite canvas workspace" aria-describedby="canvas-status" title="Use arrow keys to pan, plus/minus to zoom, and 0 to reset the view." data-ready="false" data-pan-x="0" data-pan-y="0" data-zoom="1"></canvas>
                    <div id="floating-toolbar" class="floating-toolbar" role="toolbar" aria-orientation="vertical" aria-label="Floating canvas toolbar" title="Drag the handle to reposition the toolbar." data-x="18" data-y="18">
                        <div id="floating-toolbar-handle" class="floating-toolbar__handle" aria-label="Drag handle" title="Drag handle"></div>
                    </div>
                </div>
            </section>
        </main>
        "#
    .to_string()
}

#[cfg(target_arch = "wasm32")]
pub fn install_app(
    document: &Document,
) -> Result<(HtmlElement, HtmlCanvasElement, HtmlElement, HtmlElement), JsValue> {
    let body = document
        .body()
        .ok_or_else(|| JsValue::from_str("document has no body element"))?;
    body.set_inner_html(&app_markup());

    let workspace = document
        .get_element_by_id("canvas-workspace")
        .ok_or_else(|| JsValue::from_str("canvas workspace element not found"))?
        .dyn_into::<HtmlElement>()?;
    let canvas = document
        .get_element_by_id("infinite-canvas")
        .ok_or_else(|| JsValue::from_str("canvas element not found"))?
        .dyn_into::<HtmlCanvasElement>()?;
    let status = document
        .get_element_by_id("canvas-status")
        .ok_or_else(|| JsValue::from_str("status element not found"))?
        .dyn_into::<HtmlElement>()?;
    let toolbar = document
        .get_element_by_id("floating-toolbar")
        .ok_or_else(|| JsValue::from_str("floating toolbar element not found"))?
        .dyn_into::<HtmlElement>()?;
    let toolbar_handle = document
        .get_element_by_id("floating-toolbar-handle")
        .ok_or_else(|| JsValue::from_str("floating toolbar handle element not found"))?
        .dyn_into::<HtmlElement>()?;
    toolbar.set_attribute("data-handle-id", &toolbar_handle.id())?;

    Ok((workspace, canvas, status, toolbar))
}
