use actix_web::{Responder, get};
use maud::{DOCTYPE, html};

use crate::web::{head_stuff, settings::ClientSettings};

#[get("/")]
pub async fn get(settings: ClientSettings) -> impl Responder {
    html! {
        (DOCTYPE)
        html {
            head {
                (head_stuff(&settings))
                title { "sgeargcher" }
                link rel="stylesheet" href="/assets/autocomplete.css";
                script src="/assets/autocomplete.js" defer {}
            }
            body.index-page {
                main.dont {
                    a.settings-link href="settings" { "settings" }
                    div.center-container {
                        h1 { "sgeargcher" }
                        form #search-form action="/search" method="get" {
                            div.search-input-wrapper {
                                input.search-input type="text" name="q" placeholder="sgeargch..." autofocus autocomplete="off";
                            }
                            input.search-submit type="submit" value="sgeargch";
                        }
                    }
                }
            }
        }
    }
}
