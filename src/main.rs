
#![allow(unused_imports)]
#![allow(dead_code)]

mod framebuffer;
mod maze;
mod player;
mod caster;
mod textures;
mod enemy;
mod collectable;
mod audio; 

use crate::collectable::Collectable;
use raylib::prelude::*;
use player::{Player, process_events};
use framebuffer::Framebuffer;
use maze::{Maze,load_maze};
use caster::{cast_ray, Intersect};
use std::f32::consts::PI;
use textures::TextureManager;
use enemy::{Enemy, TurnPreference, EnemyType};
use audio::AudioPlayer;
use std::time::Duration;

enum GameState { //Estados del juego
    Welcome,
    Playing,
    GameOver, 
    GameWon,
}

const TRANSPARENT_COLOR: Color = Color::new(0, 0, 0, 0);

fn draw_generic_sprite(
    framebuffer: &mut Framebuffer,
    player: &Player,
    sprite_pos: Vector2,
    sprite_texture: char,
    texture_manager: &TextureManager,
) {
    let sprite_a = (sprite_pos.y - player.pos.y).atan2(sprite_pos.x - player.pos.x);
    let mut angle_diff = sprite_a - player.a;
    while angle_diff > PI { angle_diff -= 2.0 * PI; }
    while angle_diff < -PI { angle_diff += 2.0 * PI; }

    if angle_diff.abs() > player.fov / 2.0 { return; }

    let sprite_d = player.pos.distance_to(sprite_pos);

    // AJUSTE DE DISTANCIA DE RENDERIZADO: Cambia este valor para controlar la distancia de visión
    if sprite_d < 10.0 || sprite_d > 250.0 { return; } // Rango de visión para coleccionables

    let screen_height = framebuffer.height as f32;
    let screen_width = framebuffer.width as f32;

    let sprite_size = (screen_height / sprite_d) * 70.0;
    let screen_x = ((angle_diff / player.fov) + 0.5) * screen_width;

    let start_x = (screen_x - sprite_size / 2.0).max(0.0) as usize;
    let start_y = (screen_height / 2.0 - sprite_size / 2.0).max(0.0) as usize;
    let sprite_size_usize = sprite_size as usize;
    let end_x = (start_x + sprite_size_usize).min(framebuffer.width as usize);
    let end_y = (start_y + sprite_size_usize).min(framebuffer.height as usize);

    for x in start_x..end_x {
        for y in start_y..end_y {
            let tx = ((x - start_x) * 128 / sprite_size_usize) as u32;
            let ty = ((y - start_y) * 128 / sprite_size_usize) as u32;

            let color = texture_manager.get_pixel_color(sprite_texture, tx, ty);
            
            if color != TRANSPARENT_COLOR {
                let distance_fade = (1.0 - (sprite_d / 1000.0)).max(0.0);
                let final_color = Color::new(
                    (color.r as f32 * distance_fade) as u8,
                    (color.g as f32 * distance_fade) as u8,
                    (color.b as f32 * distance_fade) as u8,
                    color.a
                );
                framebuffer.set_current_color(final_color);
                framebuffer.set_pixel(x as i32, y as i32);
            }
        }
    }
}

fn render_enemies( //Renderiza los enemigos
    framebuffer: &mut Framebuffer,
    player: &Player,
    enemies: &[Enemy],
    texture_cache: &TextureManager,
) {
    for enemy in enemies {
        draw_generic_sprite(framebuffer, player, enemy.pos, enemy.texture_key, texture_cache);
    }
}

fn render_collectables( //Renderiza los coleccionables
    framebuffer: &mut Framebuffer,
    player: &Player,
    collectables: &[Collectable],
    texture_cache: &TextureManager,
) {
    for item in collectables {
        draw_generic_sprite(framebuffer, player, item.pos, item.texture_key, texture_cache);
    }
}

fn update_enemies( //Actualiza los enemigos
    enemies: &mut Vec<Enemy>,
    delta_time: f32,
    maze: &Maze,
    block_size: usize,
) {
    for enemy in enemies {
        enemy.update(delta_time, maze, block_size);
    }
}

fn draw_cell(
    framebuffer: &mut Framebuffer,
    xo: usize,
    yo: usize,
    block_size: usize,
    cell: char,
) {
    if cell == ' ' { return; }
    framebuffer.set_current_color(Color::RED);
    for x in xo..xo + block_size {
        for y in yo..yo + block_size {
            framebuffer.set_pixel(x as i32, y as i32);
        }
    }
}

pub fn render_maze( //Renderiza el laberinto
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
    player: &Player,
) {
    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            let xo = col_index * block_size;
            let yo = row_index * block_size;
            draw_cell(framebuffer, xo, yo, block_size, cell);
        }
    }
    framebuffer.set_current_color(Color::WHITE);
    let px = player.pos.x as i32;
    let py = player.pos.y as i32;
    framebuffer.set_pixel(px, py);
    let num_rays = 20;
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = (player.a - (player.fov / 2.0)) + (player.fov * current_ray);
        cast_ray(framebuffer, &maze, &player, a, block_size, true);
    }
}


pub fn render_3d( //Renderiza el laberinto en 3D
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
    player: &Player,
    texture_cache: &TextureManager,
) {
    let num_rays = framebuffer.width;
    let hh = framebuffer.height as f32/ 2.0;

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = (player.a - (player.fov / 2.0)) + (player.fov * current_ray);
        let angle_diff = a - player.a;
        let intersect = cast_ray(framebuffer, &maze, &player, a, block_size, false);
        let d = intersect.distance;
        let c = intersect.impact;
        let corrected_distance = d * angle_diff.cos() as f32;
        let stake_height = (hh / corrected_distance)*100.0; //factor de escala rendering
        let half_stake_height = stake_height / 2.0;
        let stake_top = (hh - half_stake_height) as usize;
        let stake_bottom = (hh + half_stake_height) as usize;

        for y in stake_top..stake_bottom {
            let tx = intersect.tx;
            let ty = ((y as f32 - stake_top as f32) / (stake_bottom as f32 - stake_top as f32))*128.1; //el 128 tiene que ver con el tamaño de la textura (el ancho), cambiar tanto en main como en caster
            let color = texture_cache.get_pixel_color(c, tx as u32, ty as u32);
            let distance_fade = (1.0 - (corrected_distance / 1000.0)).max(0.0);
            let final_color = Color::new(
                (color.r as f32 * distance_fade) as u8,
                (color.g as f32 * distance_fade) as u8,
                (color.b as f32 * distance_fade) as u8,
                color.a
            );
            framebuffer.set_current_color(final_color);
            framebuffer.set_pixel(i, y as i32);
        }
    }
}

fn render_minimap(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    window_width: i32,
) {
    const MINIMAP_SCALE: f32 = 0.15;
    let map_width = (maze[0].len() as f32 * block_size as f32 * MINIMAP_SCALE) as i32;
    const BORDER_OFFSET: i32 = 10;
    let offset_x = window_width - map_width - BORDER_OFFSET;
    let offset_y = BORDER_OFFSET;
    for (j, row) in maze.iter().enumerate() {
        for (i, &cell) in row.iter().enumerate() {
            if cell != ' ' {
                let rect_x = offset_x + (i as f32 * block_size as f32 * MINIMAP_SCALE) as i32;
                let rect_y = offset_y + (j as f32 * block_size as f32 * MINIMAP_SCALE) as i32;
                let rect_w = (block_size as f32 * MINIMAP_SCALE) as i32;
                let rect_h = (block_size as f32 * MINIMAP_SCALE) as i32;
                framebuffer.set_current_color(Color::new(100, 100, 100, 180));
                for y_offset in 0..rect_h {
                    for x_offset in 0..rect_w {
                        framebuffer.set_pixel(rect_x + x_offset, rect_y + y_offset);
                    }
                }
            }
        }
    }
    let player_map_x = offset_x + (player.pos.x * MINIMAP_SCALE) as i32;
    let player_map_y = offset_y + (player.pos.y * MINIMAP_SCALE) as i32;
    framebuffer.set_current_color(Color::DARKGREEN);
    for dy in -2..=2 {
        for dx in -2..=2 {
            framebuffer.set_pixel(player_map_x + dx, player_map_y + dy);
        }
    }
    let line_length = 15.0;
    let end_x = player_map_x as f32 + line_length * player.a.cos();
    let end_y = player_map_y as f32 + line_length * player.a.sin();
    for i in 0..15 {
        let t = i as f32 / 14.0;
        let x = player_map_x as f32 * (1.0 - t) + end_x * t;
        let y = player_map_y as f32 * (1.0 - t) + end_y * t;
        framebuffer.set_pixel(x as i32, y as i32);
    }
}

fn render_welcome_screen(d: &mut RaylibDrawHandle, window_width: i32, window_height: i32) {
    // Fondo rosado oscuro
    d.clear_background(Color::new(139, 0, 139, 255)); // Purple (magenta oscuro)
    let title = "Raycaster - The Mall";
    let title_size = 50;
    let title_x = window_width / 2 - d.measure_text(title, title_size) / 2;
    // Usar colores más estilizados
    d.draw_text(title, title_x, 80, title_size, Color::PINK);
    let controls = [
        "Eres una niña en un centro comercial",
        "Encuentra todo el dinero en el nivel",
        "Cuando tengas todo el dinero, busca la bolsa para comprarla",
        "Ten cuidado en el nivel 1 que hay deuda en el centro comercial, si la topas se acaba el juego",
        "Ten cuidado en el nivel 2 que hay policia en el centro comercial, si te atrapa se acaba el juego",
        "Disfruta el juego",
        "",
        "Controles:",
        "- Moverse: W/S, Arriba/Abajo o click Izquierdo/Derecho",
        "- Girar Camara: A/D, Izquierda/Derecha o mouse",
        "- Volver al menú: Tab",
        "- Salir del juego: Esc",
    ];
    for (i, &line) in controls.iter().enumerate() {
        d.draw_text(line, 100, 200 + i as i32 * 30, 20, Color::LIGHTGRAY);
    }
    let levels = "Selecciona un nivel:";
    let levels_x = window_width / 2 - d.measure_text(levels, 30) / 2;
    d.draw_text(levels, levels_x, window_height - 250, 30, Color::GOLD);
    let easy = "[1] Deuda";
    let easy_x = window_width / 2 - d.measure_text(easy, 25) / 2;
    d.draw_text(easy, easy_x, window_height - 180, 25, Color::ORANGE);
    let hard = "[2] Policia";
    let hard_x = window_width / 2 - d.measure_text(hard, 25) / 2;
    d.draw_text(hard, hard_x, window_height - 130, 25, Color::RED);
}

fn render_game_over_screen(d: &mut RaylibDrawHandle, window_width: i32, window_height: i32) { //Pantalla de Game Over
    d.clear_background(Color::BLACK);
    let msg = "GAME OVER";
    let msg_size = 100;
    let msg_x = window_width / 2 - d.measure_text(msg, msg_size) / 2;
    d.draw_text(msg, msg_x, window_height / 2 - 100, msg_size, Color::RED);
    let restart_msg = "Presiona ENTER para volver al menú";
    let restart_size = 25;
    let restart_x = window_width / 2 - d.measure_text(restart_msg, restart_size) / 2;
    d.draw_text(restart_msg, restart_x, window_height / 2 + 50, restart_size, Color::PINK); // Texto rosa
}

fn render_win_screen(d: &mut RaylibDrawHandle, window_width: i32, window_height: i32) { //Pantalla de victoria
    d.clear_background(Color::BLACK);
    let msg = "¡Compraste la bolsa!";
    let msg_size = 100;
    let msg_x = window_width / 2 - d.measure_text(msg, msg_size) / 2;
    d.draw_text(msg, msg_x, window_height / 2 - 100, msg_size, Color::GOLD);
    let close_msg = "Presiona ENTER para cerrar el juego";
    let close_size = 25;
    let close_x = window_width / 2 - d.measure_text(close_msg, close_size) / 2;
    d.draw_text(close_msg, close_x, window_height / 2 + 50, close_size, Color::PINK); // Texto rosa
}

fn main() {
    let window_width = 1300;
    let window_height = 900;
    let block_size = 100;
    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raycaster")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();
    let texture_cache = TextureManager::new(&mut window, &raylib_thread);
    
    let mut framebuffer = Framebuffer::new(window_width, window_height, Color::BLACK);
    
    let mut game_state = GameState::Welcome;
    let mut maze: Option<Maze> = None;
    let mut player: Option<Player> = None;
    let mut enemies: Option<Vec<Enemy>> = None;
    let mut collectables: Option<Vec<Collectable>> = None;
    let mut bag_created = false; // Nuevo estado para saber si la bolsa ya ha sido creada
    let mut bag_collected = false; // Nuevo estado para saber si la bolsa ya ha sido recolectada
    let mut money_collected = 0; // Contador específico para dinero recolectado
    let mut max_money = 0;
    
    let audio_player = AudioPlayer::default();
    if let Err(e) = audio_player.play_background_music("assets/background.mp3") {
        eprintln!("Error al cargar la música de fondo: {}", e);
    }
    audio_player.set_volume(0.5);

    while !window.window_should_close() {
        match game_state {
            GameState::Welcome => {
                // Asegurar que el cursor esté habilitado en el menú
                if window.is_cursor_hidden() {
                    window.enable_cursor();
                }
                
                let mut selected_maze_file = "";
                let mut player_start_pos = Vector2::zero();
                
                if window.is_key_pressed(KeyboardKey::KEY_ONE) {
                    selected_maze_file = "maze.txt";
                    player_start_pos = Vector2::new(1.5 * block_size as f32, 6.5 * block_size as f32);
                    max_money = 6;
                    const ZERO_SPEED: f32 = 0.0;
                    enemies = Some(vec![
                        Enemy::new(2.5 * block_size as f32, 5.5 * block_size as f32, TurnPreference::Right, ZERO_SPEED, EnemyType::Debt),
                        Enemy::new(5.8 * block_size as f32, 7.7 * block_size as f32, TurnPreference::Left, ZERO_SPEED, EnemyType::Debt),
                        Enemy::new(1.6 * block_size as f32, 3.8 * block_size as f32, TurnPreference::Right, ZERO_SPEED, EnemyType::Debt),
                        Enemy::new(3.0 * block_size as f32, 1.7 * block_size as f32, TurnPreference::Left, ZERO_SPEED, EnemyType::Debt),
                        Enemy::new(7.5 * block_size as f32, 1.5 * block_size as f32, TurnPreference::Right, ZERO_SPEED, EnemyType::Debt),
                        Enemy::new(11.5 * block_size as f32, 2.4 * block_size as f32, TurnPreference::Left, ZERO_SPEED, EnemyType::Debt),
                        Enemy::new(10.6 * block_size as f32, 2.4 * block_size as f32, TurnPreference::Right, ZERO_SPEED, EnemyType::Debt),
                        Enemy::new(5.6 * block_size as f32, 3.2 * block_size as f32, TurnPreference::Left, ZERO_SPEED, EnemyType::Debt),
                        Enemy::new(4.5 * block_size as f32, 5.6 * block_size as f32, TurnPreference::Right, ZERO_SPEED, EnemyType::Debt),
                        Enemy::new(7.5 * block_size as f32, 5.5 * block_size as f32, TurnPreference::Left, ZERO_SPEED, EnemyType::Debt),
                        Enemy::new(10.6 * block_size as f32, 6.0 * block_size as f32, TurnPreference::Right, ZERO_SPEED, EnemyType::Debt),
                    ]);
                    collectables = Some(vec![
                        Collectable::new(1.5 * block_size as f32, 1.5 * block_size as f32, 'm'),
                        Collectable::new(6.5 * block_size as f32, 5.5 * block_size as f32, 'm'),
                        Collectable::new(7.5 * block_size as f32, 3.5 * block_size as f32, 'm'),
                        Collectable::new(8.0 * block_size as f32, 7.5 * block_size as f32, 'm'), //g
                        Collectable::new(11.5 * block_size as f32, 7.0 * block_size as f32, 'm'),
                        Collectable::new(1.5 * block_size as f32, 5.0 * block_size as f32, 'm'),
                    ]);
                }
                if window.is_key_pressed(KeyboardKey::KEY_TWO) {
                    selected_maze_file = "maze.txt";
                    player_start_pos = Vector2::new(1.5 * block_size as f32, 6.5 * block_size as f32);
                    max_money = 6;
                    const SPEED: f32 = 250.0;
                    enemies = Some(vec![
                        Enemy::new(1.5 * block_size as f32, 1.5 * block_size as f32, TurnPreference::Right, SPEED, EnemyType::Police),
                        Enemy::new(7.5 * block_size as f32, 1.5 * block_size as f32, TurnPreference::Left, SPEED, EnemyType::Police),
                        Enemy::new(1.5 * block_size as f32, 5.5 * block_size as f32, TurnPreference::Right, SPEED, EnemyType::Police),
                        Enemy::new(7.5 * block_size as f32, 5.5 * block_size as f32, TurnPreference::Left, SPEED, EnemyType::Police),
                        Enemy::new(1.5 * block_size as f32, 5.5 * block_size as f32, TurnPreference::Right, SPEED, EnemyType::Police),
                        Enemy::new(10.5 * block_size as f32, 6.5 * block_size as f32, TurnPreference::Left, SPEED, EnemyType::Police),
                        ]);
                    collectables = Some(vec![
                        Collectable::new(1.5 * block_size as f32, 1.5 * block_size as f32, 'm'),
                        Collectable::new(6.5 * block_size as f32, 5.5 * block_size as f32, 'm'),
                        Collectable::new(7.5 * block_size as f32, 3.5 * block_size as f32, 'm'),
                        Collectable::new(8.0 * block_size as f32, 7.5 * block_size as f32, 'm'),
                        Collectable::new(11.5 * block_size as f32, 7.0 * block_size as f32, 'm'),
                        Collectable::new(1.5 * block_size as f32, 5.0 * block_size as f32, 'm'),
                    ]);
                }

                if !selected_maze_file.is_empty() {
                    maze = Some(load_maze(selected_maze_file));
                    player = Some(Player { pos: player_start_pos, a: -PI / 2.0, fov: PI / 3.0 });
                    bag_created = false;
                    bag_collected = false;
                    money_collected = 0;
                    game_state = GameState::Playing;
                }
                let mut d = window.begin_drawing(&raylib_thread);
                render_welcome_screen(&mut d, window_width, window_height);
            }
            GameState::Playing => {
                if let (Some(p), Some(m), Some(e), Some(c)) = (&mut player, &maze, &mut enemies, &mut collectables) {
                    let delta_time = window.get_frame_time();
                    
                    framebuffer.clear();
                    
                    let half_height = (window_height / 2) as i32;
                    let floor_color = Color::new(210, 180, 140, 255);  
                    let ceiling_color = Color::new(225, 220, 215, 255);
                    
                    // Dibujar cielo (parte superior de la pantalla)
                    for y in 0..half_height {
                        for x in 0..window_width as i32 {
                            framebuffer.set_current_color(ceiling_color);
                            framebuffer.set_pixel(x, y);
                        }
                    }

                    // Dibujar tierra (parte inferior de la pantalla)
                    for y in half_height..window_height as i32 {
                        for x in 0..window_width as i32 {
                            let final_color = floor_color;
                            framebuffer.set_current_color(final_color);
                            framebuffer.set_pixel(x, y);
                        }
                    }

                    const COLLECT_DISTANCE: f32 = 35.0;
                    let mut collected_count = 0; // Contador para saber cuántos coleccionables se recolectaron
                    c.retain(|item| {
                        if p.pos.distance_to(item.pos) < COLLECT_DISTANCE {
                            collected_count += 1;
                            
                            // Distinguir entre dinero y bolsa
                            if item.texture_key == 'm' {
                                // Es dinero
                                money_collected += 1;
                            } else if item.texture_key == 'b' {
                                // Es la bolsa - cambiar al estado de victoria
                                bag_collected = true;
                                game_state = GameState::GameWon;
                                return false; // No mantener la bolsa en la lista
                            }
                            
                            false
                        } else {
                            true
                        }
                    });

                    // Reproducir sonido para cada coleccionable recolectado en este frame
                    if collected_count > 0 {
                        if let Err(e) = audio_player.play_sfx_once("assets/kaching.mp3") {
                            eprintln!("Error al reproducir sonido de coleccionable: {}", e);
                        }
                    }

                    // Si se recolectaron todos los dineros y la bolsa no ha sido creada aún
                    if money_collected == max_money && !bag_created && !bag_collected {
                        // Agregar la bolsa en la posición (8.0, 7.5)
                        c.push(Collectable::new(8.0 * block_size as f32, 7.5 * block_size as f32, 'b'));
                        bag_created = true; // Marcar que la bolsa ya ha sido creada
                    }

                    let goal_unlocked = money_collected >= max_money;
                    
                    // CAMBIO AQUÍ: Eliminamos mouse_delta_x y pasamos &mut window
                    let goal_reached = process_events(&mut window, p, m, block_size, goal_unlocked);

                    if goal_reached { game_state = GameState::GameWon; }

                    update_enemies(e, delta_time, m, block_size);
                    const COLLISION_DISTANCE: f32 = 25.0;
                    if e.iter().any(|enemy| p.pos.distance_to(enemy.pos) < COLLISION_DISTANCE) {
                        // Pausa la música, reproduce el SFX y reanuda la música al terminar
                        let _ = audio_player.play_sfx_duck_music("assets/sad.mp3", Duration::from_millis(2000)); //2000ms = 2s para que se escuche el sound effect
                        game_state = GameState::GameOver;
                    }

                    let mut mode = "3D";
                    if window.is_key_down(KeyboardKey::KEY_M) { mode = "2D"; }

                    if mode == "2D" {
                        render_maze(&mut framebuffer, m, block_size, p);
                    } else {
                        render_3d(&mut framebuffer, m, block_size, p, &texture_cache);
                        render_enemies(&mut framebuffer, p, e, &texture_cache);
                        render_collectables(&mut framebuffer, p, c, &texture_cache);
                    }
                    if mode != "2D" { render_minimap(&mut framebuffer, m, p, block_size, window_width); }
                    
                    if let Some(texture) = framebuffer.swap_buffers(&mut window, &raylib_thread) {
                        let mut d = window.begin_drawing(&raylib_thread);
                        d.clear_background(Color::BLACK);
                        d.draw_texture(&texture, 0, 0, Color::WHITE);
                        
                        let fps = d.get_fps();
                        d.draw_text(&format!("FPS: {}", fps), 10, 10, 20, Color::PINK); // Texto rosa
                        
                        let coords_text = format!("X: {:.1} Y: {:.1}", p.pos.x, p.pos.y);
                        d.draw_text(&coords_text, 10, 40, 20, Color::PINK); // Texto rosa
                        
                        // Mostrar mensaje de "Busca la bolsa" cuando se recolecten todos los dineros
                        if money_collected == max_money && bag_created && !bag_collected {
                            let msg = "¡Busca la bolsa!";
                            let msg_size = 30;
                            let msg_x = window_width / 2 - d.measure_text(msg, msg_size) / 2;
                            d.draw_text(msg, msg_x, 70, msg_size, Color::PINK); // Texto rosa
                        }
                        
                        let score_text = format!("{}/{}", money_collected, max_money);
                        let score_size = 30;
                        let score_x = window_width / 2 - d.measure_text(&score_text, score_size) / 2;
                        d.draw_text(&score_text, score_x, 10, score_size, Color::PINK); // Texto rosa
                    }
                    
                    // Permitir volver al menú con TAB - esto también habilita el cursor
                    if window.is_key_pressed(KeyboardKey::KEY_TAB) { 
                        window.enable_cursor(); // Rehabilitar cursor al volver al menú
                        game_state = GameState::Welcome; 
                    }
                }
            }
            GameState::GameOver => {
                if window.is_key_pressed(KeyboardKey::KEY_ENTER) { game_state = GameState::Welcome; }
                let mut d = window.begin_drawing(&raylib_thread);
                render_game_over_screen(&mut d, window_width, window_height);
            }
            GameState::GameWon => {
                if window.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    break;
                }
                let mut d = window.begin_drawing(&raylib_thread);
                render_win_screen(&mut d, window_width, window_height);
            }
        }
    }
}