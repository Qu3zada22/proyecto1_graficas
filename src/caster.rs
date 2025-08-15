use raylib::prelude::*;
use crate::player::Player;
use crate::maze::Maze;
use crate::framebuffer::Framebuffer;

pub struct Intersect {
    pub distance: f32,
    pub impact: char,
    pub tx: usize,
}

pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    a: f32,
    block_size: usize,
    draw: bool,
) -> Intersect {
    let mut d = 0.0;
    framebuffer.set_current_color(Color::WHITE);

    loop {
        let cos = d * a.cos();
        let sin = d * a.sin();
        let x = (player.pos.x + cos) as usize;
        let y = (player.pos.y + sin) as usize;

        let i = x / block_size;
        let j = y / block_size;
        
        if maze[j][i] != ' ' {
            let hitx = x - i * block_size;
            let hity = y - j * block_size;
            let mut maxhit = hity;

            if 1 < hitx && hitx < block_size - 1 {
                maxhit = hitx;
            }

            let x = ((maxhit as f32* 128.0) / block_size as f32) as usize; //este 128 tiene que ver con el tamaño de la textura (el ancho), cambiar tanto en main como en caster
            return Intersect {
                distance: d,
                impact: maze[j][i],
                tx: x,
            }; 
        }
        
        if draw {
            framebuffer.set_pixel(x as i32, y as i32);
        }
        
        d += 1.0;
    }
}