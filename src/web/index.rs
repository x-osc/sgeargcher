use maud::{DOCTYPE, Markup, html};

use crate::web::html_head;

pub async fn get() -> Markup {
    html! {
        (DOCTYPE)
        html {
            (html_head(None))
            body {
                h1 { "sgeargcher" }
                form #searchinput action="/search" method="get" {
                    input type="text" name="q" placeholder="sgeargch..." autofocus onfocus="this.select()" autocomplete="off";
                    input type="submit" value="sgeargch";
                }
            }
        }
    }
}
