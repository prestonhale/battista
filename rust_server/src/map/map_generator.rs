use crate::map::*;
use rand::{
    Rng,
};
use std::collections::HashMap;

const DOOR_CHANCE: f64 = 0.1;
const OPEN_ROOM_CHANCE: f64 = 0.5;

static COLORS: &'static [&str] = &["blue", "green", "red", "orange", "pink"];


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
                if map.cells[neighbor_index].room_index == map.cells[active_cell_index].room_index
                    && map.rooms[map.cells[active_cell_index].room_index].room_type
                        == RoomType::Open
                {
                    create_passage(map, &active_cell_index, &neighbor_index, &direction)
                } else {
                    create_wall(
                        map,
                        &active_cell_index,
                        Some(&get_index_from_coords(&next_coord)),
                        &direction,
                    )
                }
            }
            None => {
                let cur_cell_room_index = map.cells[active_cell_index].room_index;
                let new_neighbor_index =
                    create_cell(map, cur_cell_room_index, &next_coord, initialized_cells);
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

fn create_room(map: &mut Map) -> usize {
    let room = Room {
        color: rand_color(),
        room_type: rand_room_type(),
    };
    map.rooms.push(room);
    return map.rooms.len() - 1;
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
    map.cells[*cell_a]
        .edges
        .insert(direction.clone(), EdgeType::Wall);
    match cell_b {
        Some(cell) => {
            map.cells[*cell]
                .edges
                .insert(direction.opposite(), EdgeType::Wall);
        }
        None => (),
    };
}

fn create_door(map: &mut Map, cell_a: &usize, cell_b: &usize, direction: &MapDirection) {
    map.cells[*cell_a]
        .edges
        .insert(direction.clone(), EdgeType::Door);
    map.cells[*cell_b]
        .edges
        .insert(direction.opposite(), EdgeType::Door);
}

fn rand_color() -> String {
    let mut rng = rand::thread_rng();
    let rand_i = rng.gen_range(0..COLORS.len());
    return COLORS[rand_i].to_owned();
}

fn rand_room_type() -> RoomType {
    let room_type: RoomType = rand::random();
    return room_type;
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
    let rng_index = rng.gen_range(0..unused_directions.len());
    return unused_directions[rng_index].clone();
}


fn random_coordinate() -> Coords {
    let mut rng = rand::thread_rng();
    return Coords {
        x: rng.gen_range(0..WIDTH),
        y: rng.gen_range(0..HEIGHT),
    };
}

fn cell_is_initialized(map: &mut Map, index: &usize) -> bool {
    if map.cells[*index].edges.len() == 4 {
        return true;
    }
    return false;
}
