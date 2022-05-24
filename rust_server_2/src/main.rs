use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, RwLock};
use warp::{ws::Message, Filter, Rejection};

mod handler;
mod ws;
mod map;

type Result<T> = std::result::Result<T, Rejection>;
type Clients = Arc<RwLock<HashMap<String, Client>>>;

#[derive(Debug, Clone)]
pub struct Client {
    pub user_id: usize,
    pub topics: Vec<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));

    let (sender, receiver) = mpsc::channel(32);
    let cclients = clients.clone();
    tokio::spawn(async move{
        map::map_responder::game_loop(receiver, cclients).await;
    });

    let health_route = warp::path!("health").and_then(handler::health_handler);

    let register = warp::path("register");
    let register_routes = register
        .and(warp::post())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and(with_sender(sender.clone()))
        .and_then(handler::register_handler)
        .or(register
            .and(warp::delete())
            .and(warp::path::param())
            .and(with_clients(clients.clone()))
            .and_then(handler::unregister_handler));

    let publish = warp::path!("publish")
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(handler::publish_handler);

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(with_clients(clients.clone()))
        .and(with_sender(sender.clone()))
        .and_then(handler::ws_handler);

    let test_route = warp::path("test")
        .and(warp::get())
        .and_then(handler::test_handler);
    
    let static_assets = warp::path("static")
        .and(warp::get())
        .and(warp::fs::dir("www/static"));

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["User-Agent", "Sec-Fetch-Mode", "Referer", "Origin", "Content-Type", "Access-Control-Request-Method", "Access-Control-Request-Headers"])
        .allow_methods(vec!["POST", "GET", "OPTIONS"]);

    let routes = health_route
        .or(register_routes)
        .or(ws_route)
        .or(publish)
        .or(static_assets)
        .or(test_route)
        .with(cors);

    println!("Server started...");
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}


fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

fn with_sender(sender: map::MapSender) -> impl Filter<Extract = (map::MapSender,), Error = Infallible> + Clone {
    warp::any().map(move || sender.clone())
}

