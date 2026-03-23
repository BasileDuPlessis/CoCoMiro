use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

pub async fn health_check() -> Result<String, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init("http://localhost:3000/health", &opts)?;
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window object"))?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().map_err(|_| JsValue::from_str("Failed to cast response"))?;

    match resp.status() {
        200 => {
            let text_promise = resp.text()?;
            let text_value: JsValue = JsFuture::from(text_promise).await?;
            Ok(text_value.as_string().unwrap_or_default())
        },
        _ => Err(JsValue::from_str("Health check failed")),
    }
}