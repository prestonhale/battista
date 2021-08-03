use serde::Serialize;
use std::collections::HashMap;
use std::panic;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use crate::map::*;



#[derive(Serialize)]
pub struct MoveResponse{
    move_id: usize,
    player_id: String,
    player_position: Coords,
    cell: Cell,
    room: Room,
}

pub type ExploredCells = HashMap<usize, Cell>;

#[derive(Serialize, Clone)]
pub struct MapState{
    explored_cells: ExploredCells,
    player_state: HashMap<String, Coords>,
}


#[derive(Debug)]
pub struct RegisterResponse {
    pub player_coords: Coords,
    pub explored_cells: HashMap<usize, Cell>,
}

#[derive(Debug)]
pub enum PlayerInput{
    Register{
        user_id: String,
        resp: MapResponder<RegisterResponse>
    },

    MoveNorth{
        user_id: String,
        resp: MapResponder<Option<String>>
    },
    MoveEast{
        user_id: String,
        resp: MapResponder<Option<String>>
    },
    MoveSouth{
        user_id: String,
        resp: MapResponder<Option<String>>
    },
    MoveWest{
        user_id: String,
        resp: MapResponder<Option<String>>
    },

    Unknown,
}


pub async fn map_manager(mut rx: tokio::sync::mpsc::Receiver<PlayerInput>) {
    let map = map_generator::generate_map();
    let mut map_state = generate_map_state();

    while let Some(input) = rx.recv().await {
        println!("Received input to process: {:?}", input);
        match input {
            PlayerInput::Register {user_id, resp} => {
                match map_state.player_state.get(&user_id) {
                    Some(_) => (),
                    None => {
                        map_state.player_state.insert(
                            user_id,
                            Coords {
                                x: WIDTH / 2,
                                y: HEIGHT / 2,
                            },
                        );
                    }
                }
                let res = RegisterResponse{
                    player_coords: Coords{x: WIDTH / 2, y: HEIGHT / 2},
                    explored_cells: map_state.explored_cells.clone(),
                };
                resp.send(res).unwrap();
            }
            PlayerInput::MoveNorth { user_id, resp } => {
                let player_state = map_state.player_state.get(&user_id).unwrap().clone();
                let new_position = adjust_in_direction(&player_state, &MapDirection::North, &map);
                match new_position {
                    Some(new_position) => {
                        let next_cell = map.cells.get(get_index_from_coords(&new_position)).clone();
                        match next_cell {
                            Some(next_cell) => {
                                map_state
                                    .explored_cells
                                    .insert(get_index_from_coords(&new_position), next_cell.clone());
                                map_state.player_state.insert(user_id.clone(), new_position.clone());
    
                                let move_command = serde_json::to_string(&MoveResponse {
                                    move_id: 2,
                                    player_id: user_id,
                                    cell: next_cell.clone(),
                                    room: map.rooms[next_cell.room_index].clone(),
                                    player_position: new_position,
                                })
                                .unwrap();
                                resp.send(Some(move_command)).unwrap();
                            }
                            None => resp.send(None).unwrap(),
                        }
                    }
                    None => resp.send(None).unwrap(),
                }
            }
            PlayerInput::MoveEast { user_id, resp }  => {
                let player_state = map_state.player_state.get(&user_id).unwrap().clone();
                let new_position = adjust_in_direction(&player_state, &MapDirection::East, &map);
                match new_position {
                    Some(new_position) => {
                        let next_cell = map.cells.get(get_index_from_coords(&new_position)).clone();
                        match next_cell {
                            Some(next_cell) => {
                                map_state
                                    .explored_cells
                                    .insert(get_index_from_coords(&new_position), next_cell.clone());
                                map_state.player_state.insert(user_id.clone(), new_position.clone());
    
                                let move_command = serde_json::to_string(&MoveResponse {
                                    move_id: 2,
                                    player_id: user_id,
                                    cell: next_cell.clone(),
                                    room: map.rooms[next_cell.room_index].clone(),
                                    player_position: new_position,
                                })
                                .unwrap();
                                resp.send(Some(move_command)).unwrap();
                            }
                            None => resp.send(None).unwrap(),
                        }
                    }
                    None => resp.send(None).unwrap(),
                }
            }
            PlayerInput::MoveSouth { user_id, resp } => {
                let player_state = map_state.player_state.get(&user_id).unwrap().clone();
                let new_position = adjust_in_direction(&player_state, &MapDirection::South, &map);
                match new_position {
                    Some(new_position) => {
                        let next_cell = map.cells.get(get_index_from_coords(&new_position)).clone();
                        match next_cell {
                            Some(next_cell) => {
                                map_state
                                    .explored_cells
                                    .insert(get_index_from_coords(&new_position), next_cell.clone());
                                map_state.player_state.insert(user_id.clone(), new_position.clone());
    
                                let move_command = serde_json::to_string(&MoveResponse {
                                    move_id: 2,
                                    player_id: user_id,
                                    room: map.rooms[next_cell.room_index].clone(),
                                    cell: next_cell.clone(),
                                    player_position: new_position,
                                })
                                .unwrap();
                                resp.send(Some(move_command)).unwrap();
                            }
                            None => resp.send(None).unwrap(),
                        }
                    }
                    None => resp.send(None).unwrap(),
                }
            }
            PlayerInput::MoveWest { user_id, resp } => {
                let player_state = map_state.player_state.get(&user_id).unwrap().clone();
                let new_position = adjust_in_direction(&player_state, &MapDirection::West, &map);
                match new_position {
                    Some(new_position) => {
                        let next_cell = map.cells.get(get_index_from_coords(&new_position)).clone();
                        match next_cell {
                            Some(next_cell) => {
                                map_state
                                    .explored_cells
                                    .insert(get_index_from_coords(&new_position), next_cell.clone());
                                map_state.player_state.insert(user_id.clone(), new_position.clone());
    
                                let move_command = serde_json::to_string(&MoveResponse {
                                    move_id: 2,
                                    player_id: user_id,
                                    room: map.rooms[next_cell.room_index].clone(),
                                    cell: next_cell.clone(),
                                    player_position: new_position,
                                })
                                .unwrap();
                                resp.send(Some(move_command)).unwrap();
                            }
                            None => resp.send(None).unwrap(),
                        }
                    }
                    None => resp.send(None).unwrap(),
                }
            },
            PlayerInput::Unknown => {
                panic!("Unknown input received")
            },
        }
    }
}


pub fn generate_map_state() -> MapState {
    return MapState {
        explored_cells: HashMap::new(),
        player_state: HashMap::new(),
    };
}

// Map should only ever be mutated here
pub async fn register_player<'a> (
    mut tx: tokio::sync::mpsc::Sender<PlayerInput>,
    // tx: MapSender,
    user_id: String,
) -> RegisterResponse {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
    let player_action = PlayerInput::Register {
        user_id: user_id.clone(),
        resp: resp_tx,
    };
    tx.send(player_action).await.unwrap();
    // Potentially hang forever
    return resp_rx.await.unwrap();
}

pub async fn respond_to_player(
    // tx: tokio::sync::mpsc::Sender<PlayerInput>,
    tx: &mut MapSender,
    user_id: String,
    input: String,
) -> Option<String>{
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
    let input = match &input[..] {
        "n" => PlayerInput::MoveNorth {
            user_id: user_id.clone(),
            resp: resp_tx,
        },
        "e" => PlayerInput::MoveEast {
            user_id: user_id.clone(),
            resp: resp_tx,
        },
        "s" => PlayerInput::MoveSouth {
            user_id: user_id.clone(),
            resp: resp_tx,
        },
        "w" => PlayerInput::MoveWest {
            user_id: user_id.clone(),
            resp: resp_tx,
        },

        _ => PlayerInput::Unknown
            
    };
    
    tx.send(input).await.unwrap();
    return resp_rx.await.unwrap();

    
}

