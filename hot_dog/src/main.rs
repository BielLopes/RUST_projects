mod components;
mod backend;

use dioxus::prelude::*;

use crate::components::*;

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(NavBar)]
    #[route("/")]
    DogView,
    #[route("/favorites")]
    Favorites,
    // #[route("/:..segments")]
    // PageNotFound { segments: Vec<String> },
}

fn main() {
    dioxus::launch(App);
}

static CSS: Asset = asset!("/assets/main.css");

#[component]
pub fn App() -> Element {
    rsx! {
        document::Stylesheet { href: CSS }
        Router::<Route> {}
    }
}
