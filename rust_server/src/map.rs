use rand::Rng;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::panic;

// pub type Map = Vec<Cell>;
pub struct Map{
    cells: Vec<Cell>,
    rooms: Vec<Room>,
}

pub type MapSender = tokio::sync::mpsc::Sender<PlayerInput>;
pub type MapResponder<T> = tokio::sync::oneshot::Sender<T>;

#[derive(Serialize, Clone, Debug)]
pub struct Cell {
    room_index: usize, 
    edges: HashMap<MapDirection, EdgeType>,
}

#[derive(PartialEq, Serialize, Clone, Debug)]
pub struct Room {
    room_type: RoomType,
    color: String,
}

impl Cell {
    fn no_walls(room_index: usize) -> Cell {
        return Cell {
            room_index: room_index,
            edges: HashMap::new(),
        };
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
enum RoomType {
    Null,
    Open,
    Corridor,
}

// #[derive(Serialize)]
// pub struct MoveCommand {
//     move_id: usize,
//     cell: Cell,
// }

#[derive(Serialize)]
pub struct MoveResponse{
    move_id: usize,
    player_id: String,
    player_position: Coords,
    cell: Cell,
    room: Room,
}

#[derive(Serialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Coords {
    pub x: usize,
    pub y: usize,
}

impl fmt::Display for Coords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// impl Serialize for Coords {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//     }
// }

pub type ExploredCells = HashMap<usize, Cell>;

#[derive(Serialize, Clone)]
pub struct MapState{
    explored_cells: ExploredCells,
    player_state: HashMap<String, Coords>,
}

const WIDTH: usize = 15;
const HEIGHT: usize = 15;

#[derive(Clone, Eq, PartialEq, Serialize, Hash, Debug)]
enum MapDirection {
    North,
    East,
    South,
    West,
}

impl MapDirection {
    fn all() -> Vec<MapDirection> {
        return vec![
            MapDirection::North,
            MapDirection::East,
            MapDirection::South,
            MapDirection::West,
        ];
    }

    fn opposite(&self) -> MapDirection {
        match self {
            MapDirection::North => MapDirection::South,
            MapDirection::East => MapDirection::West,
            MapDirection::South => MapDirection::North,
            MapDirection::West => MapDirection::East,
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq)]
enum EdgeType {
    Passage,
    Wall,
    Door,
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


pub async fn map_manager<'a>(mut rx: tokio::sync::mpsc::Receiver<PlayerInput>) {
    let map = generate_map();
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

pub fn get_index_from_coords(coords: &Coords) -> usize {
    return (coords.y * WIDTH) + coords.x;
}

pub fn generate_map() -> Map {
    let cell_count = WIDTH * HEIGHT;
    let mut map = Map{
        cells: Vec::with_capacity(cell_count),
        rooms: Vec::new(),
    };
    map.rooms.push(Room{
        color: COLORS[0].to_owned(),
        room_type: RoomType::Null,
    });
    for _ in 0..cell_count {
        map.cells.push(Cell::no_walls(0));
    }
    let mut initialized_cells: HashMap<usize, bool> = HashMap::new();
    let mut active_coords: Vec<Coords> = Vec::new();
    do_first_generation_step(&mut map, &mut active_coords, &mut initialized_cells);
    while active_coords.len() > 0 {
        do_next_generation_step(&mut map, &mut active_coords, &mut initialized_cells);
    }
    println!("{}", map.rooms.len());
    return map;
}

fn random_coordinate() -> Coords {
    let mut rng = rand::thread_rng();
    return Coords {
        x: rng.gen_range(0, WIDTH),
        y: rng.gen_range(0, HEIGHT),
    };
}

fn do_first_generation_step<'a>(
    map: &'a mut Map,
    active_coords: &mut Vec<Coords>,
    initialized_cells: &mut HashMap<usize, bool>,
) {
    let start_coord = random_coordinate();
    let start_index = get_index_from_coords(&start_coord);
    create_cell(map, 0, &start_coord, initialized_cells);
    active_coords.push(start_coord);
    let new_cell = Cell::no_walls(0);
    // map.cells[start_index] = new_cell;
}

const DOOR_CHANCE: f64 = 0.1;
const OPEN_ROOM_CHANCE: f64 = 0.5;

fn do_next_generation_step(
    map: &mut Map,
    active_coords: &mut Vec<Coords>,
    initialized_cells: &mut HashMap<usize, bool>,
) {
    let active_coord = active_coords[active_coords.len() - 1].clone();
    let active_cell_index = get_index_from_coords(&active_coord);
    if cell_is_initialized(map, &active_cell_index) {
        active_coords.remove(active_coords.len() - 1);
        return;
    }
    let direction = get_random_uninitialized_direction(&map.cells[active_cell_index]);
    match adjust_in_direction(&active_coord, &direction, &map) {
        Some(next_coord) => match initialized_cells.get(&get_index_from_coords(&next_coord)) {
            Some(_) => {
                let neighbor_index = get_index_from_coords(&next_coord);
                if map.cells[neighbor_index].room_index == map.cells[active_cell_index].room_index {
                    create_passage(
                        map,
                        &active_cell_index,
                        &neighbor_index,
                        &direction,
                    )
                } else {
                    create_wall(
                        map,
                        &active_cell_index,
                        Some(&get_index_from_coords(&next_coord)),
                        &direction,
                    )
                }
            },
            None => {
                let cur_cell_room_index = map.cells[active_cell_index].room_index;
                let new_neighbor_index = create_cell(map, cur_cell_room_index, &next_coord, initialized_cells);
                let mut rng = rand::thread_rng();
                let door = rng.gen_bool(DOOR_CHANCE);
                if door {
                    create_door(map, &active_cell_index, &new_neighbor_index, &direction);
                    let new_room_index = create_room(map);
                    map.cells[new_neighbor_index].room_index = new_room_index;
                } else {
                    create_passage(map, &active_cell_index, &new_neighbor_index, &direction);
                }
                active_coords.push(next_coord);
            }
        },
        None => {
            create_wall(map, &active_cell_index, None, &direction);
        }
    }
}

fn cell_is_initialized(map: &mut Map, index: &usize) -> bool {
    if map.cells[*index].edges.len() == 4 {
        return true;
    }
    return false;
}

fn create_cell(
    map: &mut Map,
    room_index: usize,
    coord: &Coords,
    initialized_cells: &mut HashMap<usize, bool>,
) -> usize {
    let cell = Cell::no_walls(room_index);
    let i = get_index_from_coords(coord);
    map.cells[i] = cell;
    initialized_cells.insert(i, true);
    return i;
}

fn create_room(map: &mut Map) -> usize{
    let room = Room{color: rand_color(), room_type: rand_room_type()};
    map.rooms.push(room);
    return map.rooms.len() - 1;
}
    
static COLORS: &'static [&str] = &["blue", "green", "red", "orange", "pink"];

fn rand_color() -> String {
    let mut rng = rand::thread_rng();
    let rand_i = rng.gen_range(0, COLORS.len());
    return COLORS[rand_i].to_owned();
}

fn rand_room_type() -> RoomType {
    return RoomType::Corridor;
}

fn create_door(map: &mut Map, cell_a: &usize, cell_b: &usize, direction: &MapDirection) {
    map.cells[*cell_a]
        .edges
        .insert(direction.clone(), EdgeType::Door);
    map.cells[*cell_b]
        .edges
        .insert(direction.opposite(), EdgeType::Door);
}

fn create_passage(map: &mut Map, cell_a: &usize, cell_b: &usize, direction: &MapDirection) {
    map.cells[*cell_a]
        .edges
        .insert(direction.clone(), EdgeType::Passage);
    map.cells[*cell_b]
        .edges
        .insert(direction.opposite(), EdgeType::Passage);
}

fn create_wall(map: &mut Map, cell_a: &usize, cell_b: Option<&usize>, direction: &MapDirection) {
    map.cells[*cell_a].edges.insert(direction.clone(), EdgeType::Wall);
    match cell_b {
        Some(cell) => {
            map.cells[*cell]
                .edges
                .insert(direction.opposite(), EdgeType::Wall);
        }
        None => (),
    };
}

fn get_random_uninitialized_direction(active_cell: &Cell) -> MapDirection {
    let used_edges: Vec<MapDirection> = active_cell
        .edges
        .iter()
        .map(|(key, _)| key.clone())
        .collect();
    let unused_directions: Vec<MapDirection> = MapDirection::all()
        .iter()
        .filter(|v| !used_edges.contains(v))
        .cloned()
        .collect();
    let mut rng = rand::thread_rng();
    let rng_index = rng.gen_range(0, unused_directions.len());
    return unused_directions[rng_index].clone();
}

fn adjust_in_direction(active_coord: &Coords, direction: &MapDirection, map: &Map) -> Option<Coords> {
    let edges = &map.cells[get_index_from_coords(&active_coord)].edges;
    match direction {
        MapDirection::North => {
            if active_coord.y == 0
                || edges.get(&MapDirection::North).unwrap_or(&EdgeType::Passage) == &EdgeType::Wall
            {
                return None;
            };
            return Some(Coords {
                x: active_coord.x,
                y: active_coord.y - 1,
            });
        }
        MapDirection::East => {
            if active_coord.x == (WIDTH - 1) 
                || edges.get(&MapDirection::East).unwrap_or(&EdgeType::Passage) == &EdgeType::Wall
            {
                return None;
            };
            return Some(Coords {
                x: active_coord.x + 1,
                y: active_coord.y,
            });
        }
        MapDirection::South => {
            if active_coord.y == (HEIGHT - 1) 
                || edges.get(&MapDirection::South).unwrap_or(&EdgeType::Passage) == &EdgeType::Wall
            {
                return None;
            };
            return Some(Coords {
                x: active_coord.x,
                y: active_coord.y + 1,
            });
        }
        MapDirection::West => {
            if active_coord.x == 0 
                || edges.get(&MapDirection::West).unwrap_or(&EdgeType::Passage) == &EdgeType::Wall
            {
                return None;
            };
            return Some(Coords {
                x: active_coord.x - 1,
                y: active_coord.y,
            });
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

pub fn get_dimensions() -> (usize, usize) {
    return (WIDTH, HEIGHT);
}
