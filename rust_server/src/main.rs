use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, RwLock};
use warp::{ws::Message, Filter, Rejection};
use std::time::Duration;

mod handler;
mod ws;
mod map;

type Result<T> = std::result::Result<T, Rejection>;
type Clients = Arc<RwLock<HashMap<String, Client>>>;

type MapLock = Arc<RwLock<map::Map>>;
type MapStateLock = Arc<RwLock<map::MapState>>;

#[derive(Debug, Clone)]
pub struct Client {
    pub user_id: usize,
    pub topics: Vec<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));

    let map = map::generate_map();
    let map_lock = Arc::new(RwLock::new(map));

    let (tx, mut rx) = mpsc::channel(32);
    let map_manager = tokio::spawn(async move {
        map::map_manager(rx)
    });

    let player_states = map::generate_player_states();
    let player_states_lock = Arc::new(RwLock::new(player_states));
    
    let map_state = map::generate_map_state();
    let map_state_lock = Arc::new(RwLock::new(map_state));

    let health_route = warp::path!("health").and_then(handler::health_handler);

    let register = warp::path("register");
    let register_routes = register
        .and(warp::post())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and(with_tx(tx.clone()))
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
        .and(with_tx(tx.clone()))
        .and_then(handler::ws_handler);

    let test_route = warp::path("test")
        .and(warp::get())
        .and_then(handler::test_handler);

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["User-Agent", "Sec-Fetch-Mode", "Referer", "Origin", "Content-Type", "Access-Control-Request-Method", "Access-Control-Request-Headers"])
        .allow_methods(vec!["POST", "GET", "OPTIONS"]);

    let routes = health_route
        .or(register_routes)
        .or(ws_route)
        .or(publish)
        .or(test_route)
        .with(cors);

    let map_handle = map_lock.clone();
    tokio::task::spawn(async move {
        loop {
            println!("loop");
            // This block ensures the lock write lock isn't held for a whole second
            {
                let mut locked_map = map_handle.write().await;
                map::update_map(&mut(*locked_map));
            }
            let one_second = Duration::new(1, 0);
            tokio::time::delay_for(one_second).await;
            tokio::task::yield_now().await;
        }
    });

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}


fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

fn with_tx(tx: map::MapSender) -> impl Filter<Extract = (map::MapSender,), Error = Infallible> + Clone {
    warp::any().map(move || tx.clone())
}

