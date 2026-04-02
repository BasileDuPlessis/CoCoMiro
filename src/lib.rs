#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::window;

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn hello_message() -> &'static str {
    "Hello world from CoCoMiro!"
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let document = window()
        .and_then(|win| win.document())
        .ok_or_else(|| JsValue::from_str("could not access the browser document"))?;

    let body = document
        .body()
        .ok_or_else(|| JsValue::from_str("document has no body element"))?;

    let title = document.create_element("h1")?;
    title.set_text_content(Some(hello_message()));
    body.append_child(&title)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::hello_message;

    #[test]
    fn hello_message_is_correct() {
        assert_eq!(hello_message(), "Hello world from CoCoMiro!");
    }
}
