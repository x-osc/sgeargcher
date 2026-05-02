use maud::{DOCTYPE, Markup, html};

use crate::web::html_head;

pub async fn get() -> Markup {
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
