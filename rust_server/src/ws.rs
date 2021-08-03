use crate::{Client, Clients};
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
        mut tx: map::MapSender,
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

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };
        let response = client_msg(&id, msg, &mut tx, &clients).await;
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
        tx: &mut map::MapSender,
        clients: &Clients) -> Option<String>{
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

    let response = map::map_responder::respond_to_player(tx, user_id.to_string(), user_input.input).await;
    if let Some(ref some_response) = response {
        // Message is sent even on "failed" moves
        clients.read().await
            .iter()
            .for_each(|(_, client)| {
                if let Some(sender) = &client.sender {
                    let _ = sender.send(Ok(Message::text(some_response)));
                }
        });
    }

    // return response;
    return None;
}