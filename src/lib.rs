pub mod map;
pub mod combat;
use map::*;
use tcod::{colors, Color};
extern crate rand;
use rand::Rng;
use tcod::console::*;


const MAX_ROOM_MONSTERS: i32 = 3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}


//a generic object: the player, a monster, an item, the stairs
//It's always represented by a character on screen;
pub struct Object {
    pub x: i32,
    pub y: i32,
    pub char: char,
    pub color: Color,
    pub name: String,
    pub blocks: bool,
    pub alive: bool,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, name: &str, color: Color,  blocks:bool) -> Self {
        Object {
            x: x,
            y: y,
            char: char,
            color: color,
            name: name.into(),
            blocks: blocks,
            alive: false,
        }
    }


    //set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut Console){
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn set_pos(&mut self,  x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

}

fn place_objects(room: Rect, map: &Map, objects: &mut Vec<Object>) {
    //only place it if the tile is not blocked

        //generate the monster
        //choose random number of monsters
        let num_monsters = rand::thread_rng().gen_range(0, MAX_ROOM_MONSTERS + 1);

        for _ in 0..num_monsters {
            //choose random spot for this monster

            let x  = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
            let y  = rand::thread_rng().gen_range(room.y1 + 1, room.y2);
            if !is_blocked(x,y, map, objects){
                let  random = rand::thread_rng().gen_range(0, 3);

                let mut monster = if random == 0 {
                    Object::new(x, y, 'O' ,"orc" ,colors::DESATURATED_GREEN, true)
                }else if random == 1 {
                    Object::new(x, y, 'G', "goblin",colors::LIGHTER_GREEN, true)
                }else {
                    Object::new(x, y, 'P', "pyrefiend", colors::PINK, true)
                };
                monster.alive = true;
                objects.push(monster);
            }
    }


}

fn is_blocked(x: i32, y: i32, map: &Map, objects: &[Object]) -> bool {
    //first test the map tile
    if map[x as usize][y as usize].blocked {
        return true;
    }
    //now check for any blocking objects
    objects.iter().any(|object| {
        object.blocks && object.pos() == (x,y)
    })
}


//move by the given amount
pub fn move_by(id: usize, dx: i32, dy: i32, map: &Map, objects: &mut [Object]){
    let (x,y) = objects[id].pos();
    if !is_blocked(x + dx, y+ dy, map, objects){
        objects[id].set_pos(x + dx, y + dy);
    }

}