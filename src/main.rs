use std::any::Any;
use std::borrow::BorrowMut;
use std::cell::Ref;
use std::collections::HashMap;
use std::convert::Infallible;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::io;
use std::path::Path;
extern crate yaml_rust;
use yaml_rust::{YamlLoader, YamlEmitter, Yaml};
use std::env;
use std::error::Error;
use std::fs;
use std::ops::Add;
use std::ptr::{null, null_mut};
use std::sync::Arc;
use url::form_urlencoded::parse;
extern crate clap;
use clap::{Arg, App, SubCommand};
use hyper::{Body, Client, HeaderMap, Method, Request, Response, Server, StatusCode, body::to_bytes};
use hyper::service::{make_service_fn, service_fn};
use futures::TryStreamExt as _;
use hyper::body::{Buf, HttpBody};
use hyper::client::HttpConnector;
use hyper::http::uri::InvalidUri;
use tokio::io::{stdout, AsyncWriteExt as _};
use route_recognizer::Params;
use router::Router;
mod handler;
mod router;
use futures_util::future::{join, join3, join4, join5};
use futures_util::{future, TryFutureExt};
use hyper::http::HeaderValue;

#[derive(Clone, Debug)]
pub struct Location {
    location: String,
    url: String
}

pub struct ServerRginx {
    port: String,
    bind: String,
    proxy: bool,
    locations: Vec<Location>,
    name: String
}

pub struct Config{
    servers: Vec<ServerRginx>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {

    let matches = App::new("rGinx open Http proxy multi servers")
        .version("1.0")
        .author("Alex Twidl. <twidl@rginx.com>")
        .about("AS IS")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a config file")
            .takes_value(true))
        .arg(Arg::with_name("LOG")
            .help("Sets the log file to use")
            .required(false)
            .index(1))
        .arg(Arg::with_name("v")
            .short("v")
            .multiple(true)
            .help("Sets the level of verbosity"))
        .subcommand(SubCommand::with_name("test")
            .about("controls testing configuration")
            .version("1.0")
            .author("Alex Ehers. <ehers@rginx.com>")
            .arg(Arg::with_name("debug")
                .short("d")
                .help("print debug information verbosely")))
        .get_matches();
    let config = matches.value_of("config").unwrap_or("config.yml");

    let args: Vec<String> = env::args().collect();
    let path_config =format!("{}-{}",&args[0],config);
    if !Path::new(&path_config).exists() || !Path::new(&path_config).is_file() {
        panic!("encountered IO error: {}",path_config)
    }
    let contents = fs::read_to_string(path_config)
        .expect("Something went wrong reading the file");
    let configs_yml = YamlLoader::load_from_str(&contents).unwrap();
    let config_yml = &configs_yml[0];

    let mut servers:Vec<ServerRginx> = Vec::new();

    // println!("{:?}",config_yml["servers"]);
    for item in config_yml["servers"].clone() {
        // println!("{:?}",item);
        //
        // println!("{:?}", config_yml[item.as_str().unwrap().clone()]["port"]);
        let mut locations = Vec::new();

        for location in config_yml[item.as_str().unwrap().clone()]["locations"].clone() {
            locations.push(Location{
                location: location["location"].as_str().unwrap().clone().parse().unwrap(),
                url: location["url"].as_str().unwrap().clone().parse().unwrap()
            });
            println!("{:?}",location["location"].as_str().unwrap().clone());
            println!("{:?}",location["url"].as_str().unwrap().clone());
        }

        servers.push(ServerRginx{
            proxy: config_yml[item.as_str().unwrap().clone()]["proxy"].as_bool().unwrap().clone(),
            port: config_yml[item.as_str().unwrap().clone()]["port"].as_i64().unwrap().to_string(),
            bind: config_yml[item.as_str().unwrap().clone()]["bind"].as_str().unwrap().to_string(),
            locations,
            name: item.as_str().unwrap().clone().parse().unwrap()
        })
    }

    let mut configs = Config{
        servers
    };

    let mut servers_hype = Vec::new();
    for mut conf in configs.servers {
        let addr =(format!("{}:{}", &conf.bind, &conf.port)).parse().unwrap();
        let client_main = Client::new();
        let some_state = "state".to_string();
        let mut router: Router = Router::new();
        if conf.proxy {
            for location in conf.locations.clone() {
                router.get(format!("{}/{}",location.location.as_str().clone(),"*").as_str(),Box::new(handler::proxy_handler));
                router.post(format!("{}/{}",location.location.as_str().clone(),"*").as_str(),Box::new(handler::proxy_handler));
            }
        }else {
            router.get("/test/*/clone/:id", Box::new(handler::test_handler));
            router.post("/send", Box::new(handler::send_handler));
            router.get("/params/:some_param/ids", Box::new(handler::param_handler));
        }
        let shared_router = Arc::new(router);
        let mut app_state:AppState = AppState{
            state_thing: "".to_string(),
            client: client_main.clone(),
            locations: vec![],
            name: "".to_string()
        };
        if !conf.proxy {
            app_state = AppState {
                state_thing: some_state.clone(),
                client: client_main.clone(),
                locations: conf.locations.clone(),
                name: conf.name
            };
        }else {
            app_state = AppState {
                state_thing: some_state.clone(),
                client: client_main.clone(),
                locations: conf.locations.clone(),
                name: conf.name
            };
        }
        let make_svc = make_service_fn(move |_conn| {
            let router_capture = shared_router.clone();
            let app = app_state.clone();
            async move { Ok::<_, hyper::Error>(service_fn(move |mut req| route(router_capture.clone(), req, app.clone()))) }
        });
        let server = Server::bind(&addr).serve(make_svc);
        servers_hype.push(Box::new(server));
        println!("Start server to {:?} :{}", &conf.bind, &conf.port);
    }
    // Run this servers for... forever!
    future::join_all(servers_hype).await;
    Ok(())
}
#[derive(Clone, Debug)]
pub struct AppState {
    pub state_thing: String,
    pub client: Client<HttpConnector>,
    pub locations: Vec<Location>,
    pub name: String
}
async fn route(
    router: Arc<Router>,
    mut req: Request<hyper::Body>,
    app_state: AppState,
) -> Result<Response<Body>, hyper::Error> {
    let found_handler = router.route(req.uri().path(), req.method());
    let mut resp = found_handler
        .handler
        .invoke(Context::new(app_state, req, found_handler.params))
        .await;
    resp.headers_mut().insert("X-Powered-By",HeaderValue::from_static("rGinx"));
    Ok(resp)
}
#[derive(Debug)]
pub struct Context {
    pub state: AppState,
    pub req: hyper::Request<hyper::Body>,
    pub params: Params,
    body_bytes: Option<hyper::body::Bytes>,
}

impl Context {
    pub fn new(state: AppState, req: hyper::Request<hyper::Body>, params: Params) -> Context {
        Context {
            state,
            req,
            params,
            body_bytes: None,
        }
    }

    pub async fn body_json<T: serde::de::DeserializeOwned>(&mut self) -> Result<T, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let body_bytes = match self.body_bytes {
            Some(ref v) => v,
            _ => {
                let body = hyper::body::to_bytes(self.req.body_mut()).await?;
                self.body_bytes = Some(body);
                self.body_bytes.as_ref().expect("body_bytes was set above")
            }
        };
        Ok(serde_json::from_slice(&body_bytes)?)
    }
}