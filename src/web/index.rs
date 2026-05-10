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
            }
            body.index-page {
                main.dont {
                    a.settings-link href="settings" { "settings" }
                    div.center-container {
                        h1 { "sgeargcher" }
                        form #search-form action="/search" method="get" {
                            input.search-input type="text" name="q" placeholder="sgeargch..." autofocus autocomplete="off";
                            input.search-submit type="submit" value="sgeargch";
                        }
                    }
                }
            }
        }
    }
}
