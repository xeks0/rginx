use std::collections::HashMap;
use std::ops::Add;
use crate::{Context, Response};
use hyper::{Body, StatusCode};
use serde::Deserialize;

pub async fn proxy_handler(mut ctx: Context) -> Response<Body>{
    let mut response = Response::new(Body::empty());
    let mut params: HashMap<String, String> = ctx.req
        .uri()
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(HashMap::new);
    let uri = ctx.state.url;
    let uri_string = format!(
        "{}{}",
        uri,
        ctx.req.uri()
            .path_and_query()
            .map(|x| x.as_str())
            .unwrap_or("/")
    );
    let params_str:std::string::String = map_to_string(&mut params);
    let mut uri_req = String::add(uri_string.to_string(), params_str.as_str().clone()).replace(&ctx.state.location.clone(), "").parse().unwrap();
    *ctx.req.uri_mut() = uri_req;
    let mut resp_proxy = match ctx.state.client.request(ctx.req).await {
        Ok(v) => v,
        Err(e) => {
            return hyper::Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(format!("could not parse JSON: {}", e).into())
                .unwrap()
        }
    };
    println!("Response: {}", resp_proxy.status());
    let full_proxy_body = match hyper::body::to_bytes(resp_proxy.into_body()).await {
        Ok(v) => v,
        Err(e) => {
            return hyper::Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(format!("could not parse JSON: {}", e).into())
                .unwrap()
        }
    };
    *response.body_mut() = full_proxy_body.into();
    return response;
}

pub async fn test_handler(ctx: Context) -> String {
    format!("test called  as seever {}, state_thing was: {}",ctx.state.name, ctx.state.state_thing)
}
fn map_to_string(map: &mut HashMap<String, String>) -> String{
    let mut str = String::new();
    map.retain(|key, value| {
        str += &*format!("{}={}", key, value);

        !key.starts_with("a")
    });
    format!("{}", str)
}

#[derive(Deserialize)]
struct SendRequest {
    name: String,
    active: bool,
}

pub async fn send_handler(mut ctx: Context) -> Response<Body> {
    let body: SendRequest = match ctx.body_json().await {
        Ok(v) => v,
        Err(e) => {
            return hyper::Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(format!("could not parse JSON: {}", e).into())
                .unwrap()
        }
    };

    Response::new(
        format!(
            "send called with name: {} and active: {}",
            body.name, body.active
        )
            .into(),
    )
}

pub async fn param_handler(ctx: Context) -> String {
    let param = match ctx.params.find("some_param") {
        Some(v) => v,
        None => "empty",
    };
    format!("param called, param was: {}", param)
}