use rand::Rng;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;

pub type Map = Vec<Cell>;

#[derive(Serialize, Clone, Debug)]
pub struct Cell {
    color: String,
    edges: HashMap<MapDirection, EdgeType>,
}

impl Cell {
    fn no_walls(color: String) -> Cell {
        return Cell {
            color: color,
            edges: HashMap::new(),
        };
    }
}

// #[derive(Serialize)]
// pub struct MoveCommand {
//     move_id: usize,
//     cell: Cell,
// }

#[derive(Serialize)]
pub struct MoveResponse {
    move_id: usize,
    player_position: Coords,
    cell: Cell,
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

#[derive(Serialize, Clone)]
pub struct MapState {
    explored_cells: HashMap<usize, Cell>,
    player_state: HashMap<String, Coords>,
}

const WIDTH: usize = 100;
const HEIGHT: usize = 100;

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
}

#[derive(Debug)]
enum PlayerInput {
    Register,

    MoveNorth,
    MoveEast,
    MoveSouth,
    MoveWest,

    Unknown,
}

pub fn get_index_from_coords(coords: &Coords) -> usize {
    return (coords.y * WIDTH) + coords.x;
}

pub fn generate_player_states() -> HashMap<String, Coords> {
    let player_states = HashMap::new();
    return player_states;
}

pub fn generate_map() -> Vec<Cell> {
    let cell_count = WIDTH * HEIGHT;
    let mut map = Vec::with_capacity(cell_count);
    for _ in 0..cell_count {
        map.push(Cell::no_walls(String::from("grey")));
    }
    let mut initialized_cells: HashMap<usize, bool> = HashMap::new();
    let mut active_coords: Vec<Coords> = Vec::new();
    do_first_generation_step(&mut map, &mut active_coords, &mut initialized_cells);
    println!("{}", active_coords.len());
    while active_coords.len() > 0 {
        do_next_generation_step(&mut map, &mut active_coords, &mut initialized_cells);
    }
    return map;
}

fn random_coordinate() -> Coords {
    let mut rng = rand::thread_rng();
    return Coords {
        x: rng.gen_range(0, WIDTH),
        y: rng.gen_range(0, HEIGHT),
    };
}

fn do_first_generation_step(
    map: &mut Vec<Cell>,
    active_coords: &mut Vec<Coords>,
    initialized_cells: &mut HashMap<usize, bool>,
) {
    let start_coord = random_coordinate();
    let start_index = get_index_from_coords(&start_coord);
    create_cell(map, &start_coord, initialized_cells);
    active_coords.push(start_coord);
    map[start_index] = Cell::no_walls(String::from("grey"));
}

fn do_next_generation_step(
    map: &mut Vec<Cell>,
    active_coords: &mut Vec<Coords>,
    initialized_cells: &mut HashMap<usize, bool>,
) {
    let active_coord = active_coords[active_coords.len() - 1].clone();
    let active_cell_index = get_index_from_coords(&active_coord);
    if cell_is_initialized(map, &active_cell_index) {
        active_coords.remove(active_coords.len() - 1);
        return;
    }
    let direction = get_random_uninitialized_direction(&map[active_cell_index]);
    match adjust_in_direction(&active_coord, &direction, &map) {
        Some(next_coord) => match initialized_cells.get(&get_index_from_coords(&next_coord)) {
            Some(_) => create_wall(
                map,
                &active_cell_index,
                Some(&get_index_from_coords(&next_coord)),
                &direction,
            ),
            None => {
                let new_neighbor_index = create_cell(map, &next_coord, initialized_cells);
                create_passage(map, &active_cell_index, &new_neighbor_index, &direction);
                active_coords.push(next_coord);
            }
        },
        None => {
            create_wall(map, &active_cell_index, None, &direction);
        }
    }
}

fn cell_is_initialized(map: &mut Map, index: &usize) -> bool {
    if map[*index].edges.len() == 4 {
        return true;
    }
    return false;
}

fn create_cell(
    map: &mut Map,
    coord: &Coords,
    initialized_cells: &mut HashMap<usize, bool>,
) -> usize {
    let cell = Cell::no_walls(String::from("grey"));
    let i = get_index_from_coords(coord);
    map[i] = cell;
    initialized_cells.insert(i, true);
    return i;
}

fn create_passage(map: &mut Map, cell_a: &usize, cell_b: &usize, direction: &MapDirection) {
    map[*cell_a]
        .edges
        .insert(direction.clone(), EdgeType::Passage);
    map[*cell_b]
        .edges
        .insert(direction.opposite(), EdgeType::Passage);
}

fn create_wall(map: &mut Map, cell_a: &usize, cell_b: Option<&usize>, direction: &MapDirection) {
    map[*cell_a].edges.insert(direction.clone(), EdgeType::Wall);
    match cell_b {
        Some(cell) => {
            map[*cell]
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
    let edges = &map[get_index_from_coords(&active_coord)].edges;
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

fn contains_coords(coords: &Coords) -> bool {
    if coords.x >= WIDTH || coords.y >= HEIGHT {
        return false;
    }
    return true;
}

pub fn generate_map_state() -> MapState {
    return MapState {
        explored_cells: HashMap::new(),
        player_state: HashMap::new(),
    };
}

pub fn update_map(map: &mut Map) {
    map[get_index_from_coords(&Coords {
        x: WIDTH / 2,
        y: HEIGHT / 2,
    })] = Cell::no_walls(String::from("grey"))
}

// pub fn respond_to_player(player_states: &mut PlayerStates, user_id: String, input: String) -> Option<String>{
//     let player_state = player_states
//         .entry(user_id.clone())
//         .or_insert(Coords{x: 10, y: 10})
//         .clone();
//     let player_action = match &input[..] {
//         "n" => PlayerInput::MoveNorth,
//         "e" => PlayerInput::MoveEast,
//         "s" => PlayerInput::MoveSouth,
//         "w" => PlayerInput::MoveWest,
//         _ => PlayerInput::Unknown,
//     };
//     println!("Player {} took action {:?}", user_id, player_action);

//     let purple_cell = Cell{color: "purple".to_owned()};
//     let move_command = serde_json::to_string(&MoveCommand{move_id: 2, cell: purple_cell}).unwrap();
//     match player_action {
//         PlayerInput::MoveNorth => {
//             player_states.insert(user_id, Coords{x: player_state.x, y: player_state.y - 1});
//             return Some(move_command);
//         },
//         PlayerInput::MoveEast => {
//             player_states.insert(user_id, Coords{x: player_state.x + 1, y: player_state.y});
//             return Some(move_command);
//         },
//         PlayerInput::MoveSouth => {
//             player_states.insert(user_id, Coords{x: player_state.x, y: player_state.y + 1});
//             return Some(move_command);
//         },
//         PlayerInput::MoveWest => {
//             player_states.insert(user_id, Coords{x: player_state.x - 1, y: player_state.y });
//             return Some(move_command);
//         }
//         PlayerInput::Unknown => {
//             return None;
//         }
//     }

// }

// Map should only ever be mutated here
pub fn respond_to_player(
    map: &Map,
    map_state: &mut MapState,
    user_id: String,
    input: String,
) -> Option<String> {
    //This is parsing. Parsing should be in its own func.
    let player_action = match &input[..] {
        "register" => PlayerInput::Register,

        "n" => PlayerInput::MoveNorth,
        "e" => PlayerInput::MoveEast,
        "s" => PlayerInput::MoveSouth,
        "w" => PlayerInput::MoveWest,

        _ => PlayerInput::Unknown,
    };
    println!("Player {} took action {:?}", user_id, player_action);

    match player_action {
        PlayerInput::Register => {
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
            return None;
        }
        PlayerInput::MoveNorth => {
            let player_state = map_state.player_state.get(&user_id).unwrap().clone();
            let new_position = adjust_in_direction(&player_state, &MapDirection::North, &map);
            match new_position {
                Some(new_position) => {
                    let next_cell = map.get(get_index_from_coords(&new_position)).clone();
                    match next_cell {
                        Some(next_cell) => {
                            map_state
                                .explored_cells
                                .insert(get_index_from_coords(&new_position), next_cell.clone());
                            map_state.player_state.insert(user_id, new_position.clone());

                            let move_command = serde_json::to_string(&MoveResponse {
                                move_id: 2,
                                cell: next_cell.clone(),
                                player_position: new_position,
                            })
                            .unwrap();
                            return Some(move_command);
                        }
                        None => return None,
                    }
                }
                None => return None,
            }
        }
        PlayerInput::MoveEast => {
            let player_state = map_state.player_state.get(&user_id).unwrap().clone();
            let new_position = adjust_in_direction(&player_state, &MapDirection::East, &map);
            match new_position {
                Some(new_position) => {
                    let next_cell = map.get(get_index_from_coords(&new_position)).clone();
                    match next_cell {
                        Some(next_cell) => {
                            map_state
                                .explored_cells
                                .insert(get_index_from_coords(&new_position), next_cell.clone());
                            map_state.player_state.insert(user_id, new_position.clone());

                            let move_command = serde_json::to_string(&MoveResponse {
                                move_id: 2,
                                cell: next_cell.clone(),
                                player_position: new_position,
                            })
                            .unwrap();
                            return Some(move_command);
                        }
                        None => return None,
                    }
                }
                None => return None,
            }
        }
        PlayerInput::MoveSouth => {
            let player_state = map_state.player_state.get(&user_id).unwrap().clone();
            let new_position = adjust_in_direction(&player_state, &MapDirection::South, &map);
            match new_position {
                Some(new_position) => {
                    let next_cell = map.get(get_index_from_coords(&new_position)).clone();
                    match next_cell {
                        Some(next_cell) => {
                            map_state
                                .explored_cells
                                .insert(get_index_from_coords(&new_position), next_cell.clone());
                            map_state.player_state.insert(user_id, new_position.clone());

                            let move_command = serde_json::to_string(&MoveResponse {
                                move_id: 2,
                                cell: next_cell.clone(),
                                player_position: new_position,
                            })
                            .unwrap();
                            return Some(move_command);
                        }
                        None => return None,
                    }
                }
                None => return None,
            }
        }
        PlayerInput::MoveWest => {
            let player_state = map_state.player_state.get(&user_id).unwrap().clone();
            let new_position = adjust_in_direction(&player_state, &MapDirection::West, &map);
            match new_position {
                Some(new_position) => {
                    let next_cell = map.get(get_index_from_coords(&new_position)).clone();
                    match next_cell {
                        Some(next_cell) => {
                            map_state
                                .explored_cells
                                .insert(get_index_from_coords(&new_position), next_cell.clone());
                            map_state.player_state.insert(user_id, new_position.clone());

                            let move_command = serde_json::to_string(&MoveResponse {
                                move_id: 2,
                                cell: next_cell.clone(),
                                player_position: new_position,
                            })
                            .unwrap();
                            return Some(move_command);
                        }
                        None => return None,
                    }
                }
                None => return None,
            }
        }
        PlayerInput::Unknown => {
            return None;
        }
    }
}

pub fn get_player_coords(user_id: String, map_state: &mut MapState) -> Coords {
    return map_state.player_state.get(&user_id).unwrap().clone();
}

pub fn get_explored_cells(map_state: &mut MapState) -> HashMap<usize, Cell> {
    return map_state.explored_cells.clone();
}

pub fn get_dimensions() -> (usize, usize) {
    return (WIDTH, HEIGHT);
}