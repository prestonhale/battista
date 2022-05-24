use std::collections::HashMap;
use serde::Serialize;
use std::fmt;
use std::hash::Hash;
use std::time::Instant;
use std::time::Duration;

use self::map_responder::PlayerInput;
use self::map_responder::Inputs;

pub mod map_responder;
mod map_generator;

pub struct Map {
    pub cells: Vec<Cell>,
    player_state: HashMap<String, Player>,
}

impl Map {
    fn update_player_state(&mut self, inputs: Vec<PlayerInput>, frame_time: Instant) -> (Vec<String>, Vec<usize>){
        // Apply all player commands
        let mut changed_cell_indices: Vec<usize> = Vec::with_capacity(32);
        for input in inputs {
            let player : &mut Player = self.player_state.get_mut(&input.user_id).unwrap();
            let changed_cell_index = player.apply_inputs(&mut self.cells, input.input, frame_time);
            if let Some(index) = changed_cell_index {changed_cell_indices.push(index)};
        }
        
        // Apply existing state e.g. if the player is already in motion
        for (_, player) in &mut self.player_state{
            player.update(&self.cells, frame_time);
        };

        return (
            // TODO: Actually only return changed
            self.player_state.iter().map(|(user_id, _)| user_id.clone()).collect()
            , changed_cell_indices
        )
    }

    fn update_cells(&mut self) -> Vec<usize>{ 
        let mut new_cell_indices: Vec<usize> = Vec::with_capacity(32);
        for cell in &mut self.cells{
            let changed = cell.update();
            if changed {new_cell_indices.push(cell.index.clone())};
        }
        return new_cell_indices;
    }
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Player{
    user_id: String,
    coords: Coords,
    direction: MapDirection,
    #[serde(skip_serializing)]
    state: PlayerStates,
    #[serde(skip_serializing)]
    last_moved: Instant,
}

impl Player {
    fn update(&mut self, cells: &Vec<Cell>, frame_time: Instant) {
        let move_interval = Duration::new(0, 100000000);
        let mut direction_to_move: Option<MapDirection> = None;
        match self.state {
            PlayerStates::MovingNorth => { direction_to_move = Some(MapDirection::North) },
            PlayerStates::MovingEast => { direction_to_move = Some(MapDirection::East) },
            PlayerStates::MovingWest => { direction_to_move = Some(MapDirection::West) },
            PlayerStates::MovingSouth => { direction_to_move = Some(MapDirection::South) },
            _ => (),
        }
        if self.last_moved + move_interval <= frame_time {
            if let Some(direction_to_move) = direction_to_move {
                let new_coords: Option<Coords> = self.move_in_direction(cells, direction_to_move);
                if let Some(new_coords) = new_coords { self.coords = new_coords };
                self.last_moved = frame_time.clone();
            }
        }
    }

    fn apply_inputs(&mut self, cells: &mut Vec<Cell>, inputs: Inputs, frame_time: Instant) -> Option<usize>{
        // Can move once every 100ms (aka 10 times per sec)
        let move_interval = Duration::new(0, 100000000);

        // CHeck if we stopped moving
        if !inputs.north && self.state == PlayerStates::MovingNorth {self.state = PlayerStates::Idle};
        if !inputs.east && self.state == PlayerStates::MovingEast {self.state = PlayerStates::Idle};
        if !inputs.south && self.state == PlayerStates::MovingSouth {self.state = PlayerStates::Idle};
        if !inputs.west && self.state == PlayerStates::MovingWest {self.state = PlayerStates::Idle};
        if !inputs.north && !inputs.east && !inputs.south && !inputs.west {self.state = PlayerStates::Idle}

        // The "looking" state prevents a player from transitioning between a turn to look and movement
        // without first releasing all input keys
        if inputs.north {
            if self.state == PlayerStates::Idle && self.direction == MapDirection::North && self.last_moved + move_interval <= frame_time {
                let new_coords: Option<Coords> = self.move_in_direction(cells, MapDirection::North);
                if let Some(new_coords) = new_coords { self.coords = new_coords };
                self.last_moved = frame_time;
                self.state = PlayerStates::MovingNorth 
            }
            if self.state == PlayerStates::Idle {
                self.state = PlayerStates::Looking;
                self.direction = MapDirection::North;
            };
        }
        else if inputs.east {
            if self.state == PlayerStates::Idle && self.direction == MapDirection::East && self.last_moved + move_interval <= frame_time {
                let new_coords: Option<Coords> = self.move_in_direction(cells, MapDirection::East);
                if let Some(new_coords) = new_coords { self.coords = new_coords };
                self.last_moved = frame_time;
                self.state = PlayerStates::MovingEast 
            }
            if self.state == PlayerStates::Idle {
                self.state = PlayerStates::Looking;
                self.direction = MapDirection::East;
            };
        }
        else if inputs.south {
            if self.state == PlayerStates::Idle && self.direction == MapDirection::South && self.last_moved + move_interval <= frame_time {
                let new_coords: Option<Coords> = self.move_in_direction(cells, MapDirection::South);
                if let Some(new_coords) = new_coords { self.coords = new_coords };
                self.last_moved = frame_time;
                self.state = PlayerStates::MovingSouth 
            }
            if self.state == PlayerStates::Idle {
                self.state = PlayerStates::Looking;
                self.direction = MapDirection::South;
            };
        } 
        else if inputs.west {
            if self.state == PlayerStates::Idle && self.direction == MapDirection::West && self.last_moved + move_interval <= frame_time {
                let new_coords: Option<Coords> = self.move_in_direction(cells, MapDirection::West);
                if let Some(new_coords) = new_coords { self.coords = new_coords };
                self.last_moved = frame_time;
                self.state = PlayerStates::MovingWest 
            }
            if self.state == PlayerStates::Idle {
                self.state = PlayerStates::Looking;
                self.direction = MapDirection::West;
            };
        }

        if inputs.interact{
            let facing_cell_coords = adjust_in_direction(&self.coords, &self.direction, &cells);
            if let Some(cell_coords) = facing_cell_coords {
                let index = get_index_from_coords(&cell_coords);
                match cells[index].cell_type {
                    CellType::Soil => {
                        cells[index].change_type(CellType::Plant);
                        return Some(index);
                    }
                    CellType::Flower => {
                        cells[index].change_type(CellType::Soil);
                        return Some(index);
                    }
                    _ => return None
                }
            }
        }

        return None
    }

    fn move_in_direction(&self, cells: &Vec<Cell>, direction: MapDirection) -> Option<Coords> {
        let new_position = adjust_in_direction(&self.coords, &direction, cells);
        if let Some(new_position) = new_position {
            let next_cell = cells.get(get_index_from_coords(&new_position)).clone();
            if let Some(_) = next_cell { 
                return Some(new_position)
            }
        }
        return None
    }
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub enum PlayerStates {
    Idle,
    Looking,
    MovingNorth,
    MovingEast,
    MovingSouth,
    MovingWest
}

pub type MapSender = tokio::sync::mpsc::Sender<map_responder::MapRequest>;

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
enum CellType {
    Soil,
    Plant,
    Flower,
}

#[derive(Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Cell {
    index: usize,
    cell_type: CellType,
    edges: HashMap<MapDirection, EdgeType>,
    lifetime: u64,
}

impl Cell {
    fn no_walls(index: usize) -> Cell {
        return Cell {
            index: index,
            cell_type: CellType::Soil,
            edges: HashMap::new(),
            lifetime: 0,
        };
    }

    fn change_type(&mut self, cell_type: CellType){
        self.lifetime = 0;
        self.cell_type = cell_type;
    }
    
    fn clone_with_new_lifetime(&self, lifetime: u64) -> Cell{
        let mut new_cell = self.clone();
        new_cell.lifetime = lifetime;
        return new_cell;
    }

    fn update(&mut self) -> bool {
        if self.cell_type == CellType::Plant {
            self.lifetime += 1;
            if self.lifetime >= 50 {
                self.change_type(CellType::Flower);
                return true;
            }
        }
        return false;
    }
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


#[derive(PartialEq, Serialize, Clone, Debug)]
pub struct Room {
    color: String,
}

#[derive(Clone, Eq, PartialEq, Serialize, Hash, Debug)]
pub enum MapDirection {
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

#[derive(Serialize, Clone, Debug, PartialEq, Eq, Hash)]
enum EdgeType {
    Passage,
    Wall,
}

fn get_index_from_coords(coords: &Coords) -> usize {
    return coords.y * map_generator::MAP_SIDE + coords.x
}

pub fn adjust_in_direction(
    active_coord: &Coords,
    direction: &MapDirection,
    cells: &Vec<Cell>,
) -> Option<Coords> {
    let edges = &cells[get_index_from_coords(&active_coord)].edges;
    match direction {
        MapDirection::North => {
            if active_coord.y == 0
                || edges
                    .get(&MapDirection::North)
                    .unwrap_or(&EdgeType::Passage)
                    == &EdgeType::Wall
            {
                return None;
            };
            return Some(Coords {
                x: active_coord.x,
                y: active_coord.y - 1,
            });
        }
        MapDirection::East => {
            if active_coord.x == (map_generator::WIDTH - 1)
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
            if active_coord.y == (map_generator::HEIGHT - 1)
                || edges
                    .get(&MapDirection::South)
                    .unwrap_or(&EdgeType::Passage)
                    == &EdgeType::Wall
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

pub fn get_dimensions() -> (usize, usize) {
    return (map_generator::WIDTH, map_generator::HEIGHT);
}