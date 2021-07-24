use crate::{Client, Clients, MapLock, MapStateLock};
use crate::map;
use futures::{FutureExt, StreamExt};
use serde::Deserialize;
use serde_json::from_str;
use tokio::sync::mpsc;
use warp::ws::{Message, WebSocket};


#[derive(Deserialize, Debug)]
pub struct TopicsRequest {
    topics: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct UserInput {
    input: String,
}

pub async fn client_connection(
        ws: WebSocket, 
        id: String, 
        clients: Clients, 
        map: MapLock, 
        map_state: MapStateLock, 
        mut client: Client
    ) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split(); // Why is client_ws_rcv mut?
    let (client_sender, client_rcv) = mpsc::unbounded_channel();
    
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e)
        }
    }));


    client.sender = Some(client_sender);
    clients.write().await.insert(id.clone(), client);

    println!("{} connected", id);
    
    let cid = id.clone();
    let cclients = clients.clone();

    // TODO: There will be moments to push to client, e.g. a room updates under their feet
    // tokio::task::spawn(async move {
        // loop {
            // let map_lock = map.read().await;
            // let map_data = serde_json::to_string(&(*map_lock)).unwrap();
            // cclients.read().await
            //     .get(&cid).unwrap()
            //     .sender.as_ref().unwrap()
            //     .send(Ok(Message::text(map_data)))
            //     .unwrap();
            // tokio::time::delay_for(Duration::from_millis(1000)).await;
        // }
    // });

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };
        let response = client_msg(&id, msg, &clients, &map, &map_state).await;
        match response {
            Some(msg) => {
            cclients.read().await
                .get(&cid).unwrap()
                .sender.as_ref().unwrap()
                .send(Ok(Message::text(msg)))
                .unwrap();
            },
            None => (),
        }
    }

    clients.write().await.remove(&id);
    println!("{} disconnected", id);
}

async fn client_msg(
        id: &str, 
        msg: Message, 
        clients: &Clients, 
        map: &MapLock,
        map_state: &MapStateLock) -> Option<String>{
    println!("received message from {}: {:?}", id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return None,
    };

    if message == "ping" || message == "ping\n" {
        return None;
    }

    let user_input: UserInput = match from_str(&message) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error while parsing message to user input: {}", e);
            return None;
        }
    };

    let client_lock = clients.read().await;
    let user_id = client_lock.get(&id.to_string()).unwrap().user_id.clone();

    let mut locked = map_state.write().await;
    let map_lock = map.read().await;
    // let response = map::respond_to_player(&mut locked, user_id.to_string(), user_input.input);
    let response = map::respond_to_player(&map_lock, &mut locked, user_id.to_string(), user_input.input);

    return response;
}