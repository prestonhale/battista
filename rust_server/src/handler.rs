use crate::{ws, Client, Clients, Result, MapLock, MapStateLock};
use crate::map;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{http::StatusCode, reply::json, ws::Message, Reply};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    user_id: usize,
}

#[derive(Serialize, Debug)]
pub struct RegisterResponse {
    url: String,
    player_position: map::Coords,
    explored_cells: HashMap<usize, map::Cell>,
    width: usize,
    height: usize
}

#[derive(Deserialize, Debug)]
pub struct Event {
    topic: String,
    user_id: Option<usize>,
    message: String,
}

pub async fn publish_handler(body: Event, clients: Clients) -> Result<impl Reply> {
    clients
        .read()
        .await
        .iter()
        .filter(|(_, client)| match body.user_id {
            Some(v) => client.user_id == v,
            None => true,
        })
        .filter(|(_, client)| client.topics.contains(&body.topic))
        .for_each(|(_, client)| {
            if let Some(sender) = &client.sender {
                let _ = sender.send(Ok(Message::text(body.message.clone())));
            }
        });

    Ok(StatusCode::OK)
}

pub async fn register_handler(body: RegisterRequest, clients: Clients, map_state: MapStateLock, map: MapLock) -> Result<impl Reply> {
    let user_id = body.user_id;
    let uuid = Uuid::new_v4().simple().to_string();


    register_client(uuid.clone(), user_id, clients).await;
    
    let player_coords;
    let explored_cells;
    {
        let mut map_state_lock = map_state.write().await;
        let map_lock = map.read().await;
        map::respond_to_player(&map_lock, &mut map_state_lock, user_id.to_string(), "register".to_string());
        player_coords = map::get_player_coords(user_id.to_string(), &mut map_state_lock);
        explored_cells = map::get_explored_cells(&mut map_state_lock);
    }

    let (width, height) = map::get_dimensions();

    Ok(json(&RegisterResponse {
        url: format!("ws://127.0.0.1:8000/ws/{}", uuid),
        player_position: player_coords.clone(),
        explored_cells: explored_cells,
        height: height,
        width: width
    }))
}

async fn register_client(id: String, user_id: usize, clients: Clients) {
    clients.write().await.insert(
        id,
        Client {
            user_id,
            topics: vec![String::from("cats")],
            sender: None,
        },
    );
}

pub async fn unregister_handler(id: String, clients: Clients) -> Result<impl Reply> {
    clients.write().await.remove(&id);
    Ok(StatusCode::OK)
}

pub async fn ws_handler(ws: warp::ws::Ws, id: String, clients: Clients, map: MapLock, map_state: MapStateLock) -> Result<impl Reply> {
    let client = clients.read().await.get(&id).cloned();
    match client {
        Some(c) => Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, id, clients, map, map_state, c))),
        None => Err(warp::reject::not_found()),
    }
}

pub async fn test_handler() -> Result<impl Reply> {
    Ok(String::from("test"))
}

pub async fn health_handler() -> Result<impl Reply> {
    Ok(StatusCode::OK)
}