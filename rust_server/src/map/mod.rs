use std::collections::HashMap;
use serde::Serialize;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::fmt;
use std::hash::Hash;

pub mod map_responder;
mod map_generator;

pub const WIDTH: usize = 15;
pub const HEIGHT: usize = 15;

pub struct Map {
    pub cells: Vec<Cell>,
    pub rooms: Vec<Room>,
}

pub type MapSender = tokio::sync::mpsc::Sender<map_responder::PlayerInput>;
pub type MapResponder<T> = tokio::sync::oneshot::Sender<T>;

#[derive(Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Cell {
    room_index: usize,
    edges: HashMap<MapDirection, EdgeType>,
}

impl Cell {
    fn no_walls(room_index: usize) -> Cell {
        return Cell {
            room_index: room_index,
            edges: HashMap::new(),
        };
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
    room_type: RoomType,
    color: String,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
enum RoomType {
    Null,
    Open,
    Corridor,
}

impl Distribution<RoomType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RoomType {
        match rng.gen_range(0..=1) {
            0 => RoomType::Open,
            1 => RoomType::Corridor,
            _ => RoomType::Null,
        }
    }
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
    Door,
}

fn get_index_from_coords(coords: &Coords) -> usize {
    return (coords.y * WIDTH) + coords.x;
}

pub fn adjust_in_direction(
    active_coord: &Coords,
    direction: &MapDirection,
    map: &Map,
) -> Option<Coords> {
    let edges = &map.cells[get_index_from_coords(&active_coord)].edges;
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
    return (WIDTH, HEIGHT);
}