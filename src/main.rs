extern crate tcod;

use tcod::console::*;
use tcod::{colors, Color};
use tcod::input::{Key, KeyCode::*};
use roguelike_p::map::*;
use roguelike_p::*;
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
        let player = &objects[0];
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


    //draw all objects in the list
    for object in objects {
        if fov_map.is_in_fov(object.x, object.y) {
            object.draw(con);
        }

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

    //generate map (at this point its not drawn to the screen
    let (mut map, (player_x, player_y)) = make_map();
    // create an object representing the player
    let player = Object::new(player_x, player_y, '#', colors::LIGHTER_BLUE);

    //create an NPC
    let npc = Object::new(SCREEN_WIDTH / 2 - 10 , SCREEN_HEIGHT / 2 + 5, '#', colors::LIGHT_RED);

   // the list of objects
    let mut objects= [player, npc];




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

        let fov_recompute = previous_player_position != (objects[0].x, objects[0].y);
        render_all(&mut root, &mut con, &objects, &mut map, &mut fov_map, fov_recompute);

        //will draw everything on the window at once
        root.flush();

        //handles the keys and exit the game if needed
        let player = &mut objects[0];
        previous_player_position = (player.x, player.y);
        let exit = handle_keys(&mut root, player, &map);
        if exit {
            break;
        }
    }

}

fn handle_keys(root: &mut Root, player: &mut Object, map: &Map) -> bool {
    let key = root.wait_for_keypress(true);

    match key {
        Key {code: Enter, alt: true, ..} => {
            let fullscreen  = root.is_fullscreen();
            root.set_fullscreen(!fullscreen);
        }
        Key {code: Escape, ..} => return true,
        //movement keys
        Key { code: Up, ..} => player.move_by(0, -1, map),
        Key { code: Down, ..} => player.move_by(0, 1, map),
        Key { code: Left, ..} => player.move_by(-1, 0, map),
        Key { code: Right, ..} => player.move_by(1, 0, map),
        _ => {},

    }
        false

}
