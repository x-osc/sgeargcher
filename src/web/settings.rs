use actix_web::{
    FromRequest, HttpRequest, HttpResponse, HttpResponseBuilder, Responder, dev::Payload, get,
    post, web,
};
use futures::future::{Ready, ready};
use maud::{DOCTYPE, html};
use serde::Deserialize;

use crate::web::{
    AppState, head_stuff,
    utils::cookies::{get_cookie, make_cookie},
};

#[derive(Debug, Clone)]
pub struct ClientSettings {
    pub theme: String,
}

impl ClientSettings {
    pub fn from_cookies(req: &HttpRequest) -> Self {
        let default = Self::default();

        Self {
            theme: get_cookie(req, "theme").unwrap_or(default.theme),
        }
    }

    pub fn apply_to(&self, mut resp: HttpResponseBuilder) -> HttpResponseBuilder {
        resp.cookie(make_cookie("theme", self.theme.to_owned()));
        resp
    }
}

impl FromRequest for ClientSettings {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let settings = ClientSettings::from_cookies(req);
        ready(Ok(settings))
    }
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            theme: "kanagawa_dragon".into(),
        }
    }
}

#[get("/settings")]
pub async fn get(data: web::Data<AppState>, settings: ClientSettings) -> impl Responder {
    html! {
        (DOCTYPE)
        html {
            head {
                (head_stuff(&settings))
                title { "settings" }
                link rel="stylesheet" href="/assets/settings.css";
            }
            body.settings-page {
                main {
                    a.settings-back href="/" { "back" }
                    h1 { "settings" }
                    form.settings-form method="post" action="/settings" {
                        label for="theme" { "theme" }
                        select name="theme" id="theme" {
                            @for theme in &data.available_themes {
                                option
                                    value=(theme)
                                    selected[theme == &settings.theme]
                                { (theme) }
                            }
                        }
                        input.save-settings type="submit" value="save settings";
                    }
                }
            }
        }
    }
}

#[derive(Deserialize)]
struct SettingsForm {
    theme: String,
}

#[post("/settings")]
pub async fn post(data: web::Data<AppState>, form: web::Form<SettingsForm>) -> impl Responder {
    if !data.available_themes.contains(&form.theme) {
        return HttpResponse::BadRequest().body("unknown theme");
    }

    let settings = ClientSettings {
        theme: form.theme.clone(),
    };

    settings
        .apply_to(HttpResponse::SeeOther())
        .insert_header(("Location", "/settings"))
        .finish()
}
