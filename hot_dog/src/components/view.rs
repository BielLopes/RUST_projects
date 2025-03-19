use dioxus::prelude::*;

use crate::backend::save_dog;

#[derive(serde::Deserialize)]
struct DogApi {
    message: String,
}

#[component]
pub fn DogView() -> Element {
    // let mut img_src = use_signal(|| "".to_string());

    // let fetch_new = move |_| async move {
    //     let response = reqwest::get("https://dog.ceo/api/breeds/image/random")
    //         .await
    //         .unwrap()
    //         .json::<DogApi>()
    //         .await
    //         .unwrap();

    //     img_src.set(response.message);
    // };

    let mut img_src = use_resource(|| async move {
        reqwest::get("https://dog.ceo/api/breeds/image/random")
            .await
            .unwrap()
            .json::<DogApi>()
            .await
            .unwrap()
            .message
    });

    rsx! {
        div { id: "dogview",
            img { src: img_src.cloned().unwrap_or_default() }
        }
        div { id: "buttons",
            button { onclick: move |_| img_src.restart(), id: "skip", "skip" }
            button {
                id: "save",
                onclick: move |_| async move {
                    let current = img_src.cloned().unwrap();
                    img_src.restart();
                    _ = save_dog(current).await;
                },

                "save!"
            }
        }
    }
}
