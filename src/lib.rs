pub mod map;
use tcod::map::{Map as FovMap, FovAlgorithm};
use map::*;
use tcod::{colors, Color};
extern crate rand;
use rand::Rng;
use tcod::console::*;
use std::cmp;
pub const PLAYER: usize = 0;

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
    pub fighter: Option<Fighter>,
    pub ai: Option<Ai>,
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
            fighter: None,
            ai: None,
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

    //return the distance to another object
    pub fn distance_to(&self, other: &Object) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
    }

    pub fn take_damage(&mut self, damage: i32){
        // apply damage if possible
        if let Some(fighter) = self.fighter.as_mut() {
            if damage > 0 {
               fighter.hp -= damage;
            }
        }

        //check for death, call the death function {
       if let Some(fighter) = self.fighter {
           if fighter.hp <= 0 {
               self.alive = false;
               fighter.on_death.callback(self);
           }
       }
    }

    pub fn attack(&mut self, target: &mut Object){
        //a simple formula for attacck damage
        let damage = self.fighter.map_or(0, |f| f.power) - target.fighter.map_or(0, |f| f.defense);
        if damage > 0 {
            // make the target take some damage
            println!("{} damage {} for {} hit points.", self.name, target.name, damage);
            target.take_damage(damage);
        }else {
            println!("{} attacks {} but it has no effect!", self.name, target.name);
        }
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
                    let mut orc = Object::new(x, y, 'O' ,"orc" ,colors::DESATURATED_GREEN, true);
                    orc.fighter = Some(Fighter{max_hp:15, hp: 15, defense: 1, power:4, on_death: DeathCallBack::Monster});
                    orc.ai = Some(Ai);
                    orc

                }else if random == 1 {
                    //create a goblin
                   let mut goblin =  Object::new(x, y, 'G', "goblin",colors::LIGHTER_GREEN, true);
                    goblin.fighter = Some(Fighter{max_hp:10, hp: 10, defense: 0, power:3, on_death: DeathCallBack::Monster});
                    goblin.ai = Some(Ai);
                    goblin
                }else {
                    let mut pyrefiend = Object::new(x, y, 'P', "pyrefiend", colors::PINK, true);
                    pyrefiend.fighter = Some(Fighter{max_hp:6, hp: 6, defense: 0, power:2, on_death: DeathCallBack::Monster});
                    pyrefiend.ai = Some(Ai);
                    pyrefiend
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

//combat-related properties and methods (monster, player, NPC).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fighter {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
    pub on_death: DeathCallBack,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeathCallBack {
     Player,
    Monster
}

impl DeathCallBack {
    pub fn callback(self, object: &mut Object){
        use DeathCallBack::*;
        let callback: fn(&mut Object) = match self {
            Player => player_death,
            Monster => monster_death,
        };
        callback(object);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ai;

pub fn move_towards(id: usize, target_x: i32, target_y: i32, map: &Map, objects: &mut [Object]){
    //vector from tis object to the target, and distance
    let dx = target_x -  objects[id].x;
    let dy = target_y - objects[id].y;
    let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

    //normalize it to length 1 (preserving direction(, then round it and
    //convert to integer so the movement is restricted to the map grid
    let dx = (dx  as f32 / distance).round() as i32;
    let dy = (dy as f32 / distance).round() as i32;
    move_by(id, dx, dy, map, objects);
}

pub fn ai_take_turn(monster_id: usize, map: &Map, objects: &mut [Object], fov_map: &FovMap) {
    //a basic monster takes its turn. if you can see it, it can see you
    let (monster_x, monster_y) = objects[monster_id].pos();
    if fov_map.is_in_fov(monster_x, monster_y) {
        if objects[monster_id].distance_to(&objects[PLAYER]) >= 2.0 {
            //move towards player if far away
            let (player_x, player_y) = objects[PLAYER].pos();
            move_towards(monster_id, player_x, player_y, map, objects);
        } else if objects[PLAYER].fighter.map_or(false, |f| f.hp > 0) {
            //close enough, attack! (if the player is still alive.)
            let (monster, player) = mut_two(monster_id, PLAYER, objects);
            monster.attack(player);
        }
    }
}

    /// Mutably borrow two *separate* elements from the given slice
    /// Panics when the indexes are equal or out of bounds

    pub fn mut_two<T>(first_index: usize, second_index: usize, items: &mut [T]) -> (&mut T, &mut T){
        assert!(first_index != second_index);
        let split_at_index = cmp::max(first_index, second_index);
        let (first_slice, second_slice) = items.split_at_mut(split_at_index);
        if first_index < second_index {
            (&mut first_slice[first_index], &mut second_slice[0])
        }else {
            (&mut second_slice[0], &mut first_slice[second_index])
        }
    }

    pub fn player_death(player: &mut Object){
        //the game ended!
        println!("You died!");

        //for added effect, transform the player into a corpse!
        player.char = '%';
        player.color = colors::DARK_RED;
    }

    pub fn monster_death(monster: &mut Object) {
        //transform it into a nasty corpse! it doesn't block, can't be attacked and doesn't move
        println!("{} is dead!", monster.name);
        monster.char = '#';
        monster.color = colors::DARK_RED;
        monster.blocks = false;
        monster.fighter = None;
        monster.ai = None;
        monster.name = format!("remains of {}", monster.name);
    }
