use crate::components::InfiniteCanvas;
use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <InfiniteCanvas />
    }
}
