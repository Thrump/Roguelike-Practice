extern crate rand;
use rand::Rng;

use std::cmp;


//Map size
pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 45;

//room size
pub const ROOM_MAX_SIZE: i32 = 11;
pub const ROOM_MIN_SIZE: i32 = 5;
const MAX_ROOMS: i32 = 25;


#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub blocked: bool,
    pub block_sight: bool,
    pub explored: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile{blocked: false, block_sight: false, explored: false}
    }

    pub fn wall() -> Self {
        Tile {blocked: true, block_sight: true, explored: false}
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w:i32, h:i32)  -> Self {
        Rect{ x1: x, y1: y, x2: x + w, y2: y + h}
    }

    pub fn center(&self) -> (i32, i32) {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }

    pub fn intersects_with(&self, other: &Rect) -> bool {
        //returns true if this rectangle intersects with another one
        (self.x1 <= other.x2) && (self.x2 >= other.x1) &&
            (self.y1 <= other.y2) && (self.y2 >= other.y1)
    }
}

pub fn create_room(room: Rect, map: &mut Map){
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }

}

pub type Map = Vec<Vec<Tile>>;

pub fn make_map() -> (Map, (i32, i32)) {
    //fill map with "blocked tiles"
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    let mut rooms = vec![];

    let mut starting_position = (0,0);
    for _ in 0..MAX_ROOMS {
        // random width and height
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h  = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);

        //random position without going out of the boundaries of the map
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        let new_room = Rect::new(x, y, w, h);
        let failed = rooms.iter().any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            // means no intersections, so this room is valid

            //"paint" it to the map's tiles

            create_room(new_room, &mut map);

            //center coordinates of the new room, will be useful later
            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                //starting room
                starting_position = (new_x, new_y);
            }else {
                // all rooms after the first;
                //connect it to the previous room with a tunnel

                //center coordinates of the previous room
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                //toss a coin
                if rand::random() {
                    // first move horizontally, then vertically
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                }else {
                    // first move vertically, then horizontally
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                }


            }

            // finally, append the new room to the list
            rooms.push(new_room);
        }
    }
    (map, starting_position)
}

fn create_h_tunnel(x1: i32, x2: i32, y:i32, map: &mut Map) {
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1){
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(y1: i32, y2: i32, x:i32, map: &mut Map) {
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1){
        map[x as usize][y as usize] = Tile::empty();
    }
}


