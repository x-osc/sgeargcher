use std::{collections::HashMap, time::Duration};

use actix_web::{HttpRequest, HttpResponse, Responder, get, web};

use crate::{
    engine::scrapers::SearchContext,
    web::{
        AppState,
        config::{DEFAULT_USER_CONFIG, METASEARCHER},
    },
};

#[get("/complete")]
pub async fn get(
    params: web::Query<HashMap<String, String>>,
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    let Some(query) = params.get("q").map(|s| s.trim()) else {
        return HttpResponse::BadRequest().finish();
    };

    let connection_info = req.connection_info();
    let ip = connection_info
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();
    let headers: HashMap<_, _> = req
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
        .collect();

    let mut config = DEFAULT_USER_CONFIG.merge_into_default(&METASEARCHER);
    config.timeout = Duration::from_millis(data.config.timeout);

    let response = METASEARCHER
        .get_autocomplete(
            SearchContext {
                query: query.to_owned(),
                ip,
                headers,
            },
            &config,
        )
        .await;

    HttpResponse::Ok().json(response)
}
