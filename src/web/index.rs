use maud::{DOCTYPE, Markup, html};

pub async fn get() -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { "sgeargcher" }
            }
            body {
                form action="/search" method="get" {
                    input type="text" name="q" placeholder="sgeargch...";
                    button { "sgeargch" }
                }
            }
        }
    }
}
