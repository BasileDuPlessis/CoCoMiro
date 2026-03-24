mod api;
mod components;
mod constants;
mod error;
mod events;
mod rendering;
mod state;

#[allow(unused_imports)]
use crate::components::App;

#[cfg(not(test))]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
