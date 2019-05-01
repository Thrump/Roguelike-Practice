pub mod map;
use map::*;
use tcod::{colors, Color};
extern crate rand;
use rand::Rng;
use tcod::console::*;


const MAX_ROOM_MONSTERS: i32 = 3;


//a generic object: the player, a monster, an item, the stairs
//It's always represented by a character on screen;
pub struct Object {
    pub x: i32,
    pub y: i32,
    pub char: char,
    pub color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Object {
            x: x,
            y: y,
            char: char,
            color: color,
        }
    }

    //move by the given amount
    pub fn move_by(&mut self, dx: i32, dy: i32, map: &Map){
        if !map[(self.x + dx) as usize][(self.y + dy) as usize].blocked {
            self.x += dx;
            self.y += dy;
        }

    }


    //set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut Console){
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }

}

fn place_objects(room: Rect, objects: &mut Vec<Object>) {
    //choose random number of monsters
    let num_monsters = rand::thread_rng().gen_range(0, MAX_ROOM_MONSTERS + 1);

    for _ in 0..num_monsters {
        //choose random spot for this monster

        let x  = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y  = rand::thread_rng().gen_range(room.y1 + 1, room.y2);


        let mut random = rand::thread_rng().gen_range(0, 3);

        let mut monster = if random == 0 {
            Object::new(x, y, 'O' , colors::DESATURATED_GREEN)
        }else if random == 1 {
            Object::new(x, y, 'G', colors::LIGHTER_GREEN)
        }else {
            Object::new(x, y, 'P', colors::PINK)
        };
        objects.push(monster);
    }
}