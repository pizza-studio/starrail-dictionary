use log::{info, error};
use model::{NestedDictionaryItem, SearchParams};
use reqwest;
use thiserror::Error as ThisError;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::prelude::*;
use yew_hooks::use_async;

const BASE_URL: &str = "http://localhost:3001";

#[function_component]
fn App() -> Html {
    let client = use_state(|| reqwest::Client::new());

    let search_param = use_state(|| SearchParams {
        search_word: "".to_string(),
        batch_size: 10,
        page: Some(0),
    });


    let on_input = {
        let search_param = search_param.clone();
        Callback::from(move |e: Event| {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

            if let Some(input) = input {
                println!("{}", input.value());
                search_param.set(SearchParams {
                    search_word: input.value(),
                    page: search_param.page,
                    batch_size: search_param.batch_size,
                });
            }
        })
    };

    let state = {
        let search_param = search_param.clone();
        use_async(async move {
            let client = client.clone();
            let response = client
                .get(format!("{}/{}", BASE_URL, "search"))
                .fetch_mode_no_cors()
                .query(&*search_param)
                .header(reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN, "true")
                .send()
                .await
                .map_err(|err| {
                    error!("{}", err);
                    Error::RequestError
                });
            if let Ok(data) = response {
                if data.status().is_success() {
                    data.json::<Vec<NestedDictionaryItem>>()
                        .await
                        .map_err(|_| Error::DeserializeError)
                } else {
                    match data.status().as_u16() {
                        401 => Err(Error::Unauthorized),
                        403 => Err(Error::Forbidden),
                        404 => Err(Error::NotFound),
                        500 => Err(Error::InternalServerError),
                        _ => Err(Error::RequestError),
                    }
                }
            } else {
                Err(Error::RequestError)
            }
        })
    };

    let on_click = {
        let search_param_clone = search_param.clone();
        let search_param_clone2 = search_param.clone();
        let state = state.clone();
        Callback::from(move |ev: MouseEvent| {
            info!("Querying dictionary with param :{:?}", *search_param_clone);
            state.run();
        })
    };

    html! {
        <>
            <HeaderBar />
            <div>
                <input
                    onchange={on_input}
                    id="search_input"
                    type="text"
                    value={search_param.search_word.clone()}
                />
                <button onclick={on_click}>{"Search"}</button>
            </div>
            <div>
                {
                    if state.loading {
                        html!{ "loading" }
                    } else {
                        html!{}
                    }
                }
                {
                    if let Some(data) = &state.data {
                        html!{
                            <>
                                <p>{format!("Count: {}", data.len())}</p>
                                <DictionaryItemList items={data.clone()} />
                            </>
                        }
                    } else {
                        html!{}
                    }
                }
                {
                    if let Some(error) = &state.error {
                        html! { error }
                    } else {
                        html! {}
                    }
                }
            </div>
        </>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}

#[function_component(HeaderBar)]
fn header_bar() -> Html {
    html! {
        <nav class="navbar">
            <div class="container">
                <div id="navMenu" class="navbar-menu">
                    <div class="navbar-start">
                        <a class="navbar-item">
                        {"Home"}
                        </a>
                    </div>

                    <div class="navbar-end">
                        <div class="navbar-item">
                        <div class="buttons">
                            <a class="button is-dark">{"Github"}</a>
                            <a class="button is-link">{"Download"}</a>
                        </div>
                        </div>
                    </div>
                </div>
            </div>
        </nav>
    }
}

#[derive(Properties, PartialEq)]
struct DictionaryItemListProps {
    items: Vec<NestedDictionaryItem>,
}

#[function_component(DictionaryItemList)]
fn dictionary_item_list(DictionaryItemListProps { items }: &DictionaryItemListProps) -> Html {
    html! {
        <>
            {
                items.into_iter().map(|item| {
                    html!{
                        <>
                            <p>
                                { format!("{}, {}", item.target, item.target_lang) }
                            </p>
                            <p>
                                {
                                    item.clone().lan_dict.into_iter().map(|(lang, translation)| {
                                        html!{ format!{"{}: {}", lang, translation} }
                                    }).collect::<Html>()
                                }
                            </p>
                        </>
                    }
                }).collect::<Html>()
            }
        </>
    }
}

/// Define all possible errors
#[derive(ThisError, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// 401
    #[error("Unauthorized")]
    Unauthorized,

    /// 403
    #[error("Forbidden")]
    Forbidden,

    /// 404
    #[error("Not Found")]
    NotFound,

    /// 500
    #[error("Internal Server Error")]
    InternalServerError,

    /// serde deserialize error
    #[error("Deserialize Error")]
    DeserializeError,

    /// request error
    #[error("Http Request Error")]
    RequestError,
}
