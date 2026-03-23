use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use web_sys::{Request, RequestInit, RequestMode, Response};

pub async fn health_check() -> Result<String, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init("http://localhost:3000/health", &opts)?;
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window object"))?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().map_err(|_| JsValue::from_str("Failed to cast response"))?;

    match resp.status() {
        200 => {
            let text: JsValue = wasm_bindgen_futures::JsFuture::from(resp.text()?).await?;
            Ok(text.as_string().unwrap_or_default())
        },
        _ => Err(JsValue::from_str("Health check failed")),
    }
}