use actix_web::{
    HttpRequest,
    cookie::{Cookie, time::Duration},
};

pub fn make_cookie<'a>(name: &'a str, value: String) -> Cookie<'a> {
    Cookie::build(name, value)
        .path("/")
        .max_age(Duration::days(365))
        .http_only(true)
        .finish()
}

pub fn get_cookie(req: &HttpRequest, name: &str) -> Option<String> {
    req.cookie(name).map(|c| c.value().to_owned())
}
