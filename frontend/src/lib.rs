use yew::prelude::*;

mod api;

#[function_component(App)]
pub fn app() -> Html {
    let health_status = use_state(|| "Checking...".to_string());
    let has_run = use_state(|| false);

    {
        let health_status = health_status.clone();
        let has_run = has_run.clone();
        use_effect(move || {
            let cleanup = || ();
            if *has_run {
                return cleanup;
            }
            has_run.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match api::health_check().await {
                    Ok(status) => health_status.set(format!("Backend: {}", status)),
                    Err(_) => health_status.set("Backend: Unavailable".to_string()),
                }
            });
            cleanup
        });
    }

    html! {
        <div>
            <h1>{ "HELLO WORLD" }</h1>
            <p>{ (*health_status).clone() }</p>
        </div>
    }
}

#[cfg(not(test))]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    yew::Renderer::<App>::new().render();
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test(async)]
    async fn test_hello_world_display() {
        let rendered = yew::ServerRenderer::<App>::new().render().await;
        assert!(rendered.contains("HELLO WORLD"));
    }
}