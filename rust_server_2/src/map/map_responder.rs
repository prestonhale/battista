use serde::{Serialize, Deserialize};
use serde_json::json;
use tokio::time::sleep;
use std::time::{Duration, Instant};
use crate::map::*;
use crate::*;

#[derive(Serialize)]
pub struct MoveResponse {
    move_id: usize,
    player_id: String,
    player_position: Coords,
    cell: Cell,
}

#[derive(Serialize)]
pub struct CellUpdateResponse{
    cell: Cell,
    player_position: Coords,
}

#[derive(Debug)]
pub struct RegisterResponse {
    pub player_coords: Coords,
    pub explored_cells: Vec<Cell>,
}

#[derive(Debug)]
pub enum MapRequest{
    RegisterPlayer(String, tokio::sync::oneshot::Sender<RegisterResponse>),
    PlayerInput(PlayerInput),
}


#[derive(Debug)]
pub struct PlayerInput {
    pub user_id: String,
    pub input: Inputs,
}

#[derive(Debug, Deserialize)]
pub struct Inputs {
    pub north: bool,
    pub east: bool,
    pub south: bool,
    pub west: bool,
    pub interact: bool,
}

// Map modifications only ever happpen here
// Communication happens exclusively, into and out of the loop, via channels
pub async fn game_loop(
        mut map_receiver: tokio::sync::mpsc::Receiver<MapRequest>,
        clients: Clients
    ) {
    let mut map: Map = map_generator::generate_map();

    loop { 
        let frame_time = Instant::now();
        let mut player_inputs:Vec<PlayerInput> = Vec::with_capacity(32);
        let mut new_cells: Vec<Cell> = Vec::with_capacity(32);
        while let Ok(request) = map_receiver.try_recv(){
            match request {
                // Special route for sending all cells to a connecting player
                MapRequest::RegisterPlayer(user_id, resp_sender)=> {
                    match map.player_state.get(&user_id) {
                        Some(_) => (),
                        None => {
                            map.player_state.insert(
                                user_id.clone(),
                                Player{
                                    user_id: user_id.clone(),
                                    coords: Coords {
                                        x: map_generator::MAP_SIDE / 2,
                                        y: map_generator::MAP_SIDE / 2,
                                    },
                                    direction: MapDirection::North,
                                    state: PlayerStates::Idle,
                                    last_moved: Instant::now()
                                }
                            );
                        }
                    }
                    resp_sender.send(RegisterResponse{
                        player_coords: map.player_state[&user_id].coords.clone(),
                        explored_cells: map.cells.clone()}
                    ).unwrap();
                },

                // Change the player's state based on a new input
                MapRequest::PlayerInput(player_input) => {
                    player_inputs.push(player_input)
                }
            }
        }

        // TODO: Create concept of entity id
        let mut changed_player_ids: Vec<String> = Vec::with_capacity(32);
        let mut changed_cell_indices: Vec<usize> = Vec::with_capacity(32);
        
        // Update cells that change on their own
        let cell_indices = map.update_cells();
        changed_cell_indices.extend(cell_indices);

        // Update anything the player acted on
        let (player_ids, cell_indices) = map.update_player_state(player_inputs, frame_time);
        changed_player_ids.extend(player_ids);
        changed_cell_indices.extend(cell_indices);
        
        
        changed_player_ids.sort_unstable();
        changed_player_ids.dedup();
        let new_player_states: Vec<&Player> = changed_player_ids.iter().map(|user_id| map.player_state.get(user_id).unwrap()).collect();
        
        changed_cell_indices.sort_unstable();
        changed_cell_indices.dedup();
        let new_cells: Vec<&Cell> = changed_cell_indices.into_iter().map(|cell_index| &map.cells[cell_index]).collect();
        
        
        // Send changes to all players
        for (_, client) in clients.read().await.iter(){
            if let Some(sender) = &client.sender {
                if new_player_states.len() != 0 {
                    sender.send(Ok(Message::text(json!(
                        {
                            "type": "player_update",
                            "players": &new_player_states
                        }
                    ).to_string()))).unwrap();
                }
                if new_cells.len() != 0 {
                    sender.send(Ok(Message::text(json!(
                        {
                            "type": "cell_update",
                            "cells": &new_cells
                        }
                    ).to_string()))).unwrap();
                }
            }
        }
        if frame_time.elapsed().as_millis() < 33 {
            sleep(Duration::from_millis(33) - frame_time.elapsed()).await;
        }
    }
}


    // match input {
        // PlayerInput::Interact {user_id} => {
        //     let player_state: Coords = map.player_state.get(&user_id).unwrap().clone();
        //     let i: usize = get_index_from_coords(&player_state);
        //     return Some(vec!(map.cells[i].clone_with_new_type(CellType::Plant)));
        // },
        // PlayerInput::Unknown => {
        //     panic!("Unknown input received")
        // }
    // }


pub async fn register_player<'a> (
    map_sender: tokio::sync::mpsc::Sender<MapRequest>,
    user_id: String,
) -> RegisterResponse {
    let (resp_sender, resp_receiver) = tokio::sync::oneshot::channel();
    let player_action = MapRequest::RegisterPlayer (
        user_id.clone(),
        resp_sender,
    );
    map_sender.send(player_action).await.unwrap();
    // Potentially hang forever
    return resp_receiver.await.unwrap();
}

pub async fn respond_to_player(
    tx: &mut MapSender,
    user_id: String,
    message: &str,
){
    println!("{:?}", message);
    let input: Inputs =  match serde_json::from_str(&message){
        Ok(v) => v,
        Err(e) => {
            eprintln!("error while parsing message to user input: {}", e);
            return;
        }
    };
    println!("Received player input: {:?}", input);
    let player_input = PlayerInput{user_id, input};
    tx.send(MapRequest::PlayerInput(player_input)).await.unwrap();
}

