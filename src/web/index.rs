use actix_web::{Responder, get};
use maud::{DOCTYPE, html};

use crate::web::html_head;

#[get("/")]
pub async fn get() -> impl Responder {
    html! {
        (DOCTYPE)
        html {
            (html_head("sgeargcher"))
            body.index-page {
                main {
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
