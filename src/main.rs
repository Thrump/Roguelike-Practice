extern crate tcod;

use tcod::console::*;
use tcod::{colors, Color};
use tcod::input::{Key, KeyCode::*};
use roguelike_p::{map::*};
use roguelike_p::*;
use PlayerAction::*;
use PLAYER;
use tcod::map::{Map as FovMap, FovAlgorithm};

//size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 30;

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;
const TORCH_RADIUS: i32 = 10;


//wall and ground
const COLOR_DARK_WALL: Color = Color {r: 50, g: 50, b: 50};
const COLOR_LIGHT_WALL: Color = Color { r:130, g:110, b:50};
const COLOR_DARK_GROUND: Color = Color { r: 20, g: 20, b: 20};
const COLOR_LIGHT_GROUND: Color = Color { r: 200, g: 180, b: 50};


fn render_all(root: &mut Root, con: &mut Offscreen, objects: &[Object], map: &mut Map,
                fov_map: &mut FovMap, fov_recompute: bool){

    if fov_recompute {
        let player = &objects[PLAYER];
        fov_map.compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    //go through all tiles, and set their background color
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible = fov_map.is_in_fov(x, y);
            let wall = map[x as usize][y as usize].block_sight;
            let color = match (visible, wall) {
                //outside of field of view;
                (false, true) => COLOR_DARK_WALL,
                (false, false) => COLOR_DARK_GROUND,
                //inside fov:
                (true, true) => COLOR_LIGHT_WALL,
                (true, false) => COLOR_LIGHT_GROUND,
            };
            let explored = &mut map[x as usize][y as usize].explored;
            if visible {
                //since its visible, explore it
                *explored = true;
            }
            if *explored {
                //show explored tiles only (any visible tile is explored already)
                con.set_char_background(x, y, color, BackgroundFlag::Set);
            }

        }


    }

    let mut to_draw: Vec<_> = objects.iter().filter(|o| fov_map.is_in_fov(o.x, o.y)).collect();
    //sort so thatt non_blocking objects come first
    to_draw.sort_by(|o1, o2| {o1.blocks.cmp(&o2.blocks)});
    //draw the objects in the list
    for object in &to_draw {
        object.draw(con);
    }

    if let Some(fighter) = objects[PLAYER].fighter {
        root.print_ex(1, SCREEN_HEIGHT - 2, BackgroundFlag::None, TextAlignment::Left, format!("HP: {}/{}", fighter.hp, fighter.max_hp));
    }
    blit(con, (0,0), (SCREEN_WIDTH,SCREEN_HEIGHT), root, (0,0), 1.0, 1.0);

}


fn main() {
    let mut root = Root::initializer()
        .font("courier.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Rust/libtcod tutorial")
        .init();
    tcod::system::set_fps(LIMIT_FPS);

    let mut con = Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    // create an object representing the player
    let mut player = Object::new(0, 0, '#', "player",  colors::LIGHTER_BLUE, true);
    player.alive = true;
    player.fighter = Some(Fighter{max_hp:30, hp: 30, defense:2, power:5 , on_death: DeathCallBack::Player});
    // the list of objects
    let mut objects= vec![player];

    //generate map (at this point its not drawn to the screen
    let mut map = make_map(&mut objects);



    let mut fov_map = FovMap::new(MAP_WIDTH, MAP_HEIGHT);
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            fov_map.set(x, y,
            !map[x as usize][y as usize].block_sight,
            !map[x as usize][y as usize].blocked)
        }
    }

    let mut previous_player_position = (-1, -1);

    while !root.window_closed() {
        con.set_default_background(colors::BLACK);

        // clearing anything in the previous frame.
        con.clear();

        //render the screen

        let fov_recompute = previous_player_position != (objects[PLAYER].pos());
        render_all(&mut root, &mut con, &objects, &mut map, &mut fov_map, fov_recompute);

        //will draw everything on the window at once
        root.flush();

        //handles the keys and exit the game if needed
        let player = &mut objects[PLAYER];
        previous_player_position = (player.x, player.y);
        let player_action = handle_keys(&mut root, &mut objects, &map);
        if player_action == PlayerAction::Exit{
            break;
        }
        //let monsters take their turn
        if objects[PLAYER].alive && player_action != PlayerAction::DidntTakeTurn {
           for id in 0..objects.len() {
               if objects[id].ai.is_some() {
                   ai_take_turn(id, &map, &mut objects, &fov_map);
               }
           }
        }

    }

}

fn player_move_or_attack(dx: i32, dy: i32, map: &Map, objects: &mut [Object]){
    //the coordinates the player is moving to/attacking
    let x = objects[PLAYER].x + dx;
    let y = objects[PLAYER].y + dy;

    //try to find an attackable object there
    let target_id = objects.iter().position(|object| {
        object.fighter.is_some() && object.pos() == (x, y)
    });

    //attack if target found, move otherwise
    match target_id {
        Some(target_id) => {
            let (player, target) = mut_two(PLAYER, target_id, objects);
            player.attack(target);
        }
        None => {
            move_by(PLAYER, dx, dy, map, objects);
        }
    }

}

fn handle_keys(root: &mut Root, objects: &mut [Object], map: &Map) -> PlayerAction {
    let key = root.wait_for_keypress(true);

    let player_alive = objects[PLAYER].alive;
    match (key, player_alive) {
        (Key {code: Enter, alt: true, ..}, _) => {
            let fullscreen  = root.is_fullscreen();
            root.set_fullscreen(!fullscreen);
            DidntTakeTurn
        }
        (Key {code: Escape, ..}, _) => return Exit,
        //movement keys
        (Key { code: Up, ..}, true) => {
            player_move_or_attack( 0, -1, map, objects);
            TookTurn
        },
        (Key { code: Down, ..}, true) => {
            player_move_or_attack( 0, 1, map, objects);
            TookTurn
        },
        (Key { code: Left, ..}, true) => {
            player_move_or_attack( -1, 0, map, objects);
            TookTurn
        },
        (Key { code: Right, ..}, true) => {
            player_move_or_attack(1, 0, map, objects);
            TookTurn
        },
        _ => DidntTakeTurn,

    }

}
