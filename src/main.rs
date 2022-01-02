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
mod encoder;
use encoder::Encoder;
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

pub struct ServerRginx {
    port: String,
    bind: String,
    proxy: bool,
    url: String,
    location: String,
    name: String
}

pub struct Config{
    servers: Vec<ServerRginx>,
}
async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
//
// async fn hello_world(mut req: Request<Body>, client:Client<HttpConnector>) -> Result<Response<Body>, hyper::Error> {
//     println!("{:?}",req.headers());
//     let mut response = Response::new(Body::empty());
//     match (req.method(), req.uri().path()) {
//         (&Method::GET, "/") => {
//             *response.body_mut() = Body::from("Try POSTing data to /echo such as: `curl localhost:8080/echo -XPOST -d 'hello world'");
//         },
//         (&Method::POST, "/echo") => {
//             *response.body_mut() = req.into_body();
//         },
//         (&Method::POST, "/echo/uppercase") => {
//             // This is actually a new `futures::Stream`...
//             let mapping = req
//                 .into_body()
//                 .map_ok(|chunk| {
//                     chunk.iter()
//                         .map(|byte| byte.to_ascii_uppercase())
//                         .collect::<Vec<u8>>()
//                 });
//
//             // Use `Body::wrap_stream` to convert it to a `Body`...
//             *response.body_mut() = Body::wrap_stream(mapping);
//         },
//         (&Method::POST, "/echo/reverse") => {
//             // Await the full body to be concatenated into a single `Bytes`...
//             let full_body = hyper::body::to_bytes(req.into_body()).await?;
//
//             // Iterate the full body in reverse order and collect into a new Vec.
//             let reversed = full_body.iter()
//                 .rev()
//                 .cloned()
//                 .collect::<Vec<u8>>();
//
//             *response.body_mut() = reversed.into();
//         },
//         (&Method::GET, "/proxy") => {
//             let mut params: HashMap<String, String> = req
//                 .uri()
//                 .query()
//                 .map(|v| {
//                     url::form_urlencoded::parse(v.as_bytes())
//                         .into_owned()
//                         .collect()
//                 })
//                 .unwrap_or_else(HashMap::new);
//             let uri = "httpbin.org/ip";
//             let uri_string = format!(
//                 "http://{}{}",
//                 uri,
//                 req.uri()
//                     .path_and_query()
//                     .map(|x| x.as_str())
//                     .unwrap_or("/")
//             );
//             let params_str = map_to_string(&mut params);
//             let uri = uri_string.add(&*params_str).replace("/proxy", "").parse().unwrap();
//             *req.uri_mut() = uri;
//             let mut resp_proxy = client.request(req).await?;
//             println!("Response: {}", resp_proxy.status());
//             let full_proxy_body = hyper::body::to_bytes(resp_proxy.into_body()).await?;
//             *response.body_mut() = full_proxy_body.into();
//
//         },
//         _ => {
//             *response.status_mut() = StatusCode::NOT_FOUND;
//         },
//     };
//
//
//     Ok(response)
// }
// fn map_to_string(map: &mut HashMap<String, String>) -> String{
//     let mut str = String::new();
//     map.retain(|key, value| {
//         str += &*format!("{}={}", key, value);
//
//         !key.starts_with("a")
//     });
//    return str;
// }
// fn handle_connection(mut stream: TcpStream, configs: &Config) {
//     let file_name = get_path(&stream);
//     let file_path = format!("{}{}",configs.path, &file_name);
//     if !Path::new(&file_path).exists() || !Path::new(&file_path).is_file() {
//         let mut content_type =  "Content-type: text/html";
//         let headers =
//             ["HTTP/1.1 404 OK", content_type,"\r\n"];
//         let mut response: Vec<u8> = headers.join("\r\n")
//             .to_string()
//             .into_bytes();
//
//         // response.extend("File Not found".chars());
//         match stream.write(&response) {
//             Ok(_) => println!("Response sent"),
//             Err(e) => println!("Failed sending response: {}", e),
//         }
//         return;
//     }
//     let mut buf = Vec::new();
//     let path = Path::new(&file_name);
//
//     let mut file = File::open(&file_path).unwrap();
//     let mut content_type =  "Content-type: text/html";
//     if path.extension().unwrap() == "png" {
//          content_type =  "Content-type: image/png";
//     }
//     if path.extension().unwrap() == "jpg" || path.extension().unwrap() == "jpeg" {
//         content_type =  "Content-type: image/jpeg";
//     }
//     let headers =
//         ["HTTP/1.1 200 OK", content_type,"Transfer-Encoding: chunked","\r\n"];
//     file.read_to_end(&mut buf).unwrap();
//     let mut encoded = Vec::new();
//     {
//         let mut encoder = Encoder::with_chunks_size(&mut encoded, buf.len()/8192);
//         encoder.write_all(&buf).unwrap();
//     }
//
//     let mut response: Vec<u8> = headers.join("\r\n")
//         .to_string()
//         .into_bytes();
//     response.extend(encoded.clone());
//     match stream.write(&response) {
//         Ok(_) => println!("Response sent"),
//         Err(e) => println!("Failed sending response: {}", e),
//     }
// }
// fn get_extension_from_filename(filename: &str) -> Option<&str> {
//     Path::new(filename)
//         .extension()
//         .and_then(OsStr::to_str)
// }
// fn get_path(mut stream: &TcpStream) -> String {
//     let mut buf = [0u8; 4096];
//     match stream.read(&mut buf) {
//         Ok(_) => {
//             let req_str = String::from_utf8_lossy(&buf);
//             let path: Vec<&str> = req_str.lines().next().unwrap().split(" ").collect();
//             if path.len()<2 {
//              return "".parse().unwrap();
//             }
//             println!("GET {}", path[1]);
//             // println!("{}", req_str);
//             return path[1].to_string()
//         }
//         Err(e) => {
//             println!("Unable to read stream: {}", e);
//             "/".to_string()
//         }
//     }
// }
fn wait_for_fd(){

}
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let matches = App::new("rGinx open Http image server")
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
        servers.push(ServerRginx{
            proxy: config_yml[item.as_str().unwrap().clone()]["proxy"].as_bool().unwrap().clone(),
            port: config_yml[item.as_str().unwrap().clone()]["port"].as_i64().unwrap().to_string(),
            bind: config_yml[item.as_str().unwrap().clone()]["bind"].as_str().unwrap().to_string(),
            url: config_yml[item.as_str().unwrap().clone()]["url"].as_str().unwrap_or("").to_string(),
            location: config_yml[item.as_str().unwrap().clone()]["location"].as_str().unwrap_or("/").to_string(),
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
            router.get(format!("{}/{}",conf.location.as_str().clone(),":some_param").as_str(),Box::new(handler::proxy_handler));
            router.post(format!("{}/{}",conf.location.as_str().clone(),":some_param").as_str(),Box::new(handler::proxy_handler));
        }else {
            router.get("/test", Box::new(handler::test_handler));
            router.post("/send", Box::new(handler::send_handler));
            router.get("/params/:some_param/ids", Box::new(handler::param_handler));
        }
        let shared_router = Arc::new(router);
        let mut app_state:AppState = AppState{
            state_thing: "".to_string(),
            client: client_main.clone(),
            url: "1".to_string(),
            location: "".to_string(),
            name: "".to_string()
        };
        if !conf.proxy {
            app_state = AppState {
                state_thing: some_state.clone(),
                client: client_main.clone(),
                url: "2".to_string(),
                location: conf.location,
                name: conf.name
            };
        }else {
            app_state = AppState {
                state_thing: some_state.clone(),
                client: client_main.clone(),
                url: conf.url.clone(),
                location: conf.location,
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
    pub url: String,
    pub location: String,
    pub name: String
}
async fn route(
    router: Arc<Router>,
    req: Request<hyper::Body>,
    app_state: AppState,
) -> Result<Response<Body>, hyper::Error> {
    let found_handler = router.route(req.uri().path(), req.method());
    let resp = found_handler
        .handler
        .invoke(Context::new(app_state, req, found_handler.params))
        .await;
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