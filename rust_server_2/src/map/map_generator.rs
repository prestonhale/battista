use crate::map::*;

pub const PLOT_SIDE: usize = 20;


pub const PLOT_SIZE: usize = PLOT_SIDE.pow(2);
pub const MAP_SIDE: usize = PLOT_SIDE * 3;
pub const HEIGHT: usize = MAP_SIDE;
pub const WIDTH: usize = MAP_SIDE;
pub const MAP_SIZE: usize = 9 * PLOT_SIZE;

pub fn generate_map() -> Map {
    let mut map = Map{
        cells: Vec::with_capacity(MAP_SIZE),
        player_state: HashMap::new(),
    };
    for index in 0..HEIGHT * WIDTH {
        let cell: Cell = Cell::no_walls(index);
        map.cells.push(cell);
        let y: usize = index / MAP_SIDE;
        let x: usize = index - (MAP_SIDE * y);
        if y % PLOT_SIDE == 0 { // Northern edge
            create_wall(&mut map, &index, None, &MapDirection::North);
        }
        if x % PLOT_SIDE == PLOT_SIDE - 1 { // Eastern edge
            create_wall(&mut map, &index, None, &MapDirection::East);
        }
        if y % PLOT_SIDE == PLOT_SIDE - 1{ // Southern edge
            create_wall(&mut map, &index, None, &MapDirection::South);
        }
        if x % PLOT_SIDE == 0{ // Western edge
            create_wall(&mut map, &index, None, &MapDirection::West);
        } 
    }
    println!("Map cells {}", map.cells.len());
    return map;
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
