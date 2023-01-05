use crate::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(not(target_arch = "wasm32"))]
use rayon::{slice::ParallelSliceMut, iter::{IndexedParallelIterator, ParallelIterator}};

pub struct Raycaster {
    time                    : u128,
    old_time                : u128,

    move_speed              : f32,
    rot_speed               : f32,

    pos                     : vec2,
    dir                     : vec2,
    plane                   : vec2,

    anim_curr_time          : u128,
    anim_time               : u128,
    anim_counter            : usize,
}

impl Raycaster {

    pub fn new() -> Self {

        Self {
            pos             : vec2::new(5.0, 5.0),
            dir             : vec2::new(-1.0, 0.0),
            plane           : vec2::new(0.0, 0.66),

            time            : 0,
            old_time        : 0,

            move_speed      : 0.0,
            rot_speed       : 0.0,

            anim_curr_time  : 0,
            anim_time       : 250,
            anim_counter    : 0,
        }
    }

    #[cfg(feature = "single_threaded")]
    pub fn render(&mut self, frame: &mut [u8], rect: (usize, usize, usize, usize), stride: usize, world: &WorldMap) {
        self.render_st(frame, rect, stride, world);
    }

    #[cfg(not(feature = "single_threaded"))]
    pub fn render(&mut self, frame: &mut [u8], rect: (usize, usize, usize, usize), stride: usize, world: &WorldMap) {
        self.render_mt(frame, rect, stride, world);
    }

    /// Renders the world map into the frame inside the given rectangle
    pub fn render_st(&mut self, frame: &mut [u8], rect: (usize, usize, usize, usize), stride: usize, world: &WorldMap) {

        let start = self.get_time();

        // Update animation counter every anim_time milliseconds
        if self.anim_curr_time > self.anim_time {
            self.anim_curr_time -= self.anim_time;
            self.anim_counter = self.anim_counter.wrapping_add(1);
        }

        let width = rect.2 as i32;
        let height = rect.3 as i32;

        let pos = self.pos.clone();
        let dir = self.dir.clone();
        let plane = self.plane.clone();

        let ceiling_tile = world.get_ceiling_tile();
        let mut ceiling_is_textured = false;

        let floor_tile = world.get_floor_tile();
        let mut floor_is_textured = false;

        // Background color if no ceiling or floor tile is set
        if ceiling_tile.is_none() || floor_tile.is_none() {
            for y in rect.1..rect.3 {
                for x in rect.0..rect.2 {
                    frame[(y*stride+x)*4+0] = 0;
                    frame[(y*stride+x)*4+1] = 0;
                    frame[(y*stride+x)*4+2] = 0;
                    frame[(y*stride+x)*4+3] = 255;
                }
            }
        }

        // Ceiling color
        if let Some(ceiling) = ceiling_tile {
            if let Some(color) = ceiling.color {
                for y in rect.1..rect.3 / 2 {
                    for x in rect.0..rect.2 {
                        let o = (y*stride+x)*4;
                        frame[o..o+4].copy_from_slice(&color);
                    }
                }
            } else {
                ceiling_is_textured = true;
            }
        }

        // Floor color
        if let Some(floor) = floor_tile {
            if let Some(color) = floor.color {
                for y in rect.1 + rect.3 / 2..rect.1 + rect.3 {
                    for x in rect.0..rect.2 {
                        let o = (y*stride+x)*4;
                        frame[o..o+4].copy_from_slice(&color);
                    }
                }
            } else {
                floor_is_textured = true;
            }
        }

        // Texture the ceiling and floor

        if ceiling_is_textured || floor_is_textured {

            for y in rect.1 + rect.3 / 2..rect.3 {

                // rayDir for leftmost ray (x = 0) and rightmost ray (x = w)
                let ray_dir_x0 = dir.x - plane.x;
                let ray_dir_y0 = dir.y - plane.y;
                let ray_dir_x1 = dir.x + plane.x;
                let ray_dir_y1 = dir.y + plane.y;

                // Current y position compared to the center of the screen (the horizon)
                let p = y - rect.3 / 2;

                // Vertical position of the camera.
                let pos_z = 0.5 * rect.3 as f32;

                // Horizontal distance from the camera to the floor for the current row.
                // 0.5 is the z position exactly in the middle between floor and ceiling.
                let row_distance = pos_z / p as f32;

                // calculate the real world step vector we have to add for each x (parallel to camera plane)
                // adding step by step avoids multiplications with a weight in the inner loop
                let floor_step_x = row_distance * (ray_dir_x1 - ray_dir_x0) / rect.2 as f32;
                let floor_step_y = row_distance * (ray_dir_y1 - ray_dir_y0) / rect.2 as f32;

                // real world coordinates of the leftmost column. This will be updated as we step to the right.
                let mut floor_x = pos.x + row_distance * ray_dir_x0;
                let mut floor_y = pos.y + row_distance * ray_dir_y0;

                let mix_factor = row_distance / world.fog_distance;

                for x in rect.0..rect.2 {

                    // the cell coord is simply got from the integer parts of floorX and floorY
                    let cell_x = floor_x.floor();
                    let cell_y = floor_y.floor();

                    if let Some(floor) = floor_tile {
                        if let Some((image_id, rect)) = floor.texture {
                            let tex_x = ((rect.2 as f32 * (floor_x - cell_x)) as usize).clamp(0, rect.2 - 1);
                            let tex_y = ((rect.3 as f32 * (floor_y - cell_y)) as usize).clamp(0, rect.3 - 1);

                            if let Some((tex_data, tex_width, _tex_height)) = world.get_image(image_id) {
                                let tex_off = rect.0 + tex_x * 4 + rect.1 + ((tex_y as usize) * *tex_width as usize * 4);
                                let off = x * 4 + y * 4 * stride;

                                if mix_factor <= 0.0 {
                                    frame[off..off+4].copy_from_slice(&tex_data[tex_off..tex_off+4]);
                                } else
                                if mix_factor >= 1.0 {
                                    frame[off..off+4].copy_from_slice(&world.fog_color);
                                } else {
                                    let mut floor_color : [u8;4] = [0, 0, 0, 0];
                                    floor_color.copy_from_slice(&tex_data[tex_off..tex_off+4]);
                                    let color = self.mix_color(&floor_color, &world.fog_color, mix_factor);
                                    frame[off..off+4].copy_from_slice(&color);
                                }
                            }
                        }
                    }

                    if let Some(ceiling) = ceiling_tile {
                        if let Some((image_id, tex_rect)) = ceiling.texture {
                            let tex_x = ((tex_rect.2 as f32 * (floor_x - cell_x)) as usize).clamp(0, tex_rect.2 - 1);
                            let tex_y = ((tex_rect.3 as f32 * (floor_y - cell_y)) as usize).clamp(0, tex_rect.3 - 1);

                            if let Some((tex_data, tex_width, _tex_height)) = world.get_image(image_id) {
                                let tex_off = tex_rect.0 + tex_x * 4 + tex_rect.1 + ((tex_y as usize) * *tex_width as usize * 4);
                                let off = x * 4 + (rect.3 - y - 1) * 4 * stride;

                                if mix_factor <= 0.0 {
                                    frame[off..off+4].copy_from_slice(&tex_data[tex_off..tex_off+4]);
                                } else
                                if mix_factor >= 1.0 {
                                    frame[off..off+4].copy_from_slice(&world.fog_color);
                                } else {
                                    let mut ceiling_color : [u8;4] = [0, 0, 0, 0];
                                    ceiling_color.copy_from_slice(&tex_data[tex_off..tex_off+4]);
                                    let color = self.mix_color(&ceiling_color, &world.fog_color, mix_factor);
                                    frame[off..off+4].copy_from_slice(&color);
                                }
                            }
                        }
                    }

                    floor_x += floor_step_x;
                    floor_y += floor_step_y;
                }
            }
        }

        // Render the walls

        let mut z_buffer = vec![f32::MAX; rect.2 as usize];

        for x in rect.0..rect.2 {

            let camera_x = 2.0 * x as f32 / width as f32 - 1.0; //x-coordinate in camera space
            let ray_dir_x = dir.x + plane.x * camera_x;
            let ray_dir_y = dir.y + plane.y * camera_x;

            // which box of the map we're in
            let mut map_x = pos.x as i32;
            let mut map_y = pos.y as i32;

            // length of ray from current position to next x or y-side
            let mut side_dist_x;
            let mut side_dist_y;

            // length of ray from one x or y-side to next x or y-side
            let delta_dist_x = if ray_dir_x == 0.0 { f32::MAX } else { (1.0 / ray_dir_x).abs() };
            let delta_dist_y = if ray_dir_y == 0.0 { f32::MAX } else { (1.0 / ray_dir_y).abs() };
            let perp_wall_dist;

            // what direction to step in x or y-direction (either +1 or -1)
            let step_x;
            let step_y;

            let mut hit = false; //was there a wall hit?
            let mut side = 0; //was a NS or a EW wall hit?

            // calculate step and initial sideDist
            if ray_dir_x < 0.0 {
                step_x = -1;
                side_dist_x = (pos.x - map_x as f32) * delta_dist_x;
            } else {
                step_x = 1;
                side_dist_x = (map_x as f32 + 1.0 - pos.x) * delta_dist_x;
            }

            if ray_dir_y < 0.0 {
                step_y = -1;
                side_dist_y = (pos.y - map_y as f32) * delta_dist_y;
            } else {
                step_y = 1;
                side_dist_y = (map_y as f32 + 1.0 - pos.y) * delta_dist_y;
            }

            // perform DDA
            for _ in 0..40 {
                // jump to next map square, either in x-direction, or in y-direction
                if side_dist_x < side_dist_y {
                    side_dist_x += delta_dist_x;
                    map_x += step_x;
                    side = 0;
                } else {
                    side_dist_y += delta_dist_y;
                    map_y += step_y;
                    side = 1;
                }

                // check if ray has hit a wall
                if world.has_wall(map_x, map_y) == true {
                    hit = true;
                    break;
                }
            }

            if hit {

                // calculate distance projected on camera direction (Euclidean distance would give fisheye effect!)
                if side == 0 {
                    perp_wall_dist = side_dist_x - delta_dist_x;
                } else {
                    perp_wall_dist = side_dist_y - delta_dist_y;
                }

                // calculate height of line to draw on screen
                let line_height = (height as f32 / perp_wall_dist) as i32;

                // calculate lowest and highest pixel to fill in current stripe
                let mut draw_start = -line_height / 2 + height / 2;
                if draw_start < 0 {
                    draw_start = 0;
                }

                let mut draw_end = line_height / 2 + height / 2;
                if draw_end >= height {
                    draw_end = height - 1;
                }

                let mix_factor = perp_wall_dist / world.fog_distance;

                if let Some(tile) = world.get_wall(map_x, map_y) {

                    if let Some((image_id, rect)) = self.get_texture(&tile) {

                        // texturing calculations

                        // calculate value of wall_x
                        let mut wall_x; //where exactly the wall was hit
                        if side == 0 {
                            wall_x = pos.y + perp_wall_dist * ray_dir_y;
                        } else {
                            wall_x = pos.x + perp_wall_dist * ray_dir_x;
                        }
                        wall_x -= wall_x.floor();

                        // x coordinate on the texture
                        let mut tex_x = (wall_x * rect.2 as f32) as usize;
                        if side == 0 && ray_dir_x > 0.0 {
                            tex_x = rect.2 - tex_x - 1;
                        }
                        if side == 1 && ray_dir_y < 0.0 {
                            tex_x = rect.2 - tex_x - 1;
                        }

                        // How much to increase the texture coordinate per screen pixel
                        let step = 1.0 * rect.3 as f32 / line_height as f32;

                        let mut tex_pos = (draw_start - height / 2 + line_height / 2) as f32 * step;

                        if let Some((tex_data, tex_width, _tex_height)) = world.get_image(image_id) {
                            let off_x = x * 4;
                            for y in draw_start..draw_end {
                                let tex_off = rect.0 + tex_x * 4 + rect.1 + ((tex_pos as usize) * *tex_width as usize * 4);
                                let off = off_x + y as usize * 4 * stride;

                                if mix_factor <= 0.0 {
                                    frame[off..off+4].copy_from_slice(&tex_data[tex_off..tex_off+4]);
                                } else
                                if mix_factor >= 1.0 {
                                    frame[off..off+4].copy_from_slice(&world.fog_color);
                                } else {
                                    let mut wall_color : [u8;4] = [0, 0, 0, 0];
                                    wall_color.copy_from_slice(&tex_data[tex_off..tex_off+4]);
                                    let color = self.mix_color(&wall_color, &world.fog_color, mix_factor);
                                    frame[off..off+4].copy_from_slice(&color);
                                }

                                tex_pos += step;
                            }
                        }
                        /*
                        // Starting texture coordinate
                        double texPos = (drawStart - h / 2 + lineHeight / 2) * step;
                        for(int y = drawStart; y<drawEnd; y++)
                        {
                            // Cast the texture coordinate to integer, and mask with (texHeight - 1) in case of overflow
                            int texY = (int)texPos & (texHeight - 1);
                            texPos += step;
                            Uint32 color = texture[texNum][texHeight * texY + texX];
                            //make color darker for y-sides: R, G and B byte each divided through two with a "shift" and an "and"
                            if(side == 1) color = (color >> 1) & 8355711;
                            buffer[y][x] = color;
                        }*/
                    }
                }

                // perpendicular distance is stored in the z-buffer for sprite casting
                z_buffer[x] = perp_wall_dist;

                /* color
                let off_x = x * 4;

                for y in draw_start..draw_end {
                    let off = off_x + y as usize * 4 * self.width;
                    frame[off] = 255;
                    frame[off + 1] = 255;
                    frame[off + 2] = 255;
                    frame[off + 3] = 255;
                }
                */
            }
        }

        // Render the sprites

        for sprite in &world.sprites {

            // translate sprite position to relative to camera
            let sprite_x = sprite.x - pos.x;
            let sprite_y = sprite.y - pos.y;

            // transform sprite with the inverse camera matrix
            // [ planeX   dirX ] -1                                       [ dirY      -dirX ]
            // [               ]       =  1/(planeX*dirY-dirX*planeY) *   [                 ]
            // [ planeY   dirY ]                                          [ -planeY  planeX ]

            let inv_det = 1.0 / (plane.x * dir.y - dir.x * plane.y); //required for correct matrix multiplication

            let transform_x = inv_det * (dir.y * sprite_x - dir.x * sprite_y);
            let transform_y = inv_det * (-plane.y * sprite_x + plane.x * sprite_y); //this is actually the depth inside the screen, that what Z is in 3D

            let mix_factor = transform_y / world.fog_distance;

            let sprite_screen_x = ((width as f32 / 2.0) * (1.0 + transform_x / transform_y)) as i32;

            // calculate height of the sprite on screen
            let sprite_height = ((height as f32 / (transform_y)) as i32).abs(); //using 'transformY' instead of the real distance prevents fisheye
            // calculate lowest and highest pixel to fill in current stripe
            let mut draw_start_y = -sprite_height / 2 + height / 2;
            if draw_start_y < 0 { draw_start_y = 0; }
            let mut draw_end_y = sprite_height / 2 + height / 2;
            if draw_end_y >= height { draw_end_y = height - 1; }

            // calculate width of the sprite
            let sprite_width = ((height as f32 / (transform_y)) as i32).abs();
            let mut draw_start_x = -sprite_width / 2 + sprite_screen_x;
            if draw_start_x < 0 { draw_start_x = 0; }
            let mut draw_end_x = sprite_width / 2 + sprite_screen_x;
            if draw_end_x >= width { draw_end_x = width - 1; }

            if let Some((image_id, tex_rect)) = self.get_texture(&sprite.tile) {
                if let Some((tex_data, tex_width, _tex_height)) = world.get_image(image_id) {

                    // loop through every vertical stripe of the sprite on screen
                    for stripe in draw_start_x..draw_end_x {
                        let tex_x = ((256 * (stripe - (-sprite_width / 2 + sprite_screen_x)) * tex_rect.2 as i32 / sprite_width) / 256) as usize;

                        // the conditions in the if are:
                        // 1) it's in front of camera plane so you don't see things behind you
                        // 2) it's on the screen (left)
                        // 3) it's on the screen (right)
                        // 4) ZBuffer, with perpendicular distance

                        if transform_y > 0.0 && stripe > 0 && stripe < width && transform_y < z_buffer[stripe as usize] {
                            for y in draw_start_y as usize .. draw_end_y as usize {

                                let d = (y) * 256 - height as usize * 128 + sprite_height as usize * 128; //256 and 128 factors to avoid floats
                                let tex_y = (((d * tex_rect.3) / sprite_height as usize) / 256) as usize;

                                let tex_off = tex_rect.0 + tex_x * 4 + tex_rect.1 + ((tex_y as usize) * *tex_width as usize * 4);
                                let off = (rect.0 + stripe as usize) * 4 + (rect.1 + y as usize) * 4 * stride;

                                if mix_factor <= 0.0 {
                                    frame[off..off+4].copy_from_slice(&tex_data[tex_off..tex_off+4]);
                                } else
                                if mix_factor >= 1.0 {
                                    frame[off..off+4].copy_from_slice(&world.fog_color);
                                } else {
                                    let mut wall_color : [u8;4] = [0, 0, 0, 0];
                                    wall_color.copy_from_slice(&tex_data[tex_off..tex_off+4]);
                                    let tex_alpha = tex_data[tex_off+3] as f32 / 255.0;
                                    if tex_alpha > 0.0 {
                                        let color = self.mix_color(&wall_color, &world.fog_color, mix_factor * tex_alpha);
                                        frame[off..off+4].copy_from_slice(&color);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let stop = self.get_time();
        println!("tick time {:?}", stop - start);

        self.old_time = self.time;
        self.time = self.get_time();

        if self.old_time > 0 {
            self.anim_curr_time += self.time - self.old_time;
        }

        let frame_time = (self.time - self.old_time) as f32 / 1000.0;
        // println!("fps {}", 1.0 / frame_time); //FPS counter

        self.move_speed = frame_time * 5.0; //the constant value is in squares/second
        self.rot_speed = frame_time * 2.0;
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Renders the world map into the frame inside the given rectangle
    pub fn render_mt(&mut self, frame: &mut [u8], in_rect: (usize, usize, usize, usize), in_stride: usize, world: &WorldMap) {

        let rect = (0, 0, in_rect.2, in_rect.3);
        let stride = rect.3;

        let start = self.get_time();

        // Update animation counter every anim_time milliseconds
        if self.anim_curr_time > self.anim_time {
            self.anim_curr_time -= self.anim_time;
            self.anim_counter = self.anim_counter.wrapping_add(1);
        }

        let width = rect.2 as i32;
        let height = rect.3 as i32;

        let pos = self.pos.clone();
        let dir = self.dir.clone();
        let plane = self.plane.clone();

        let mut buffer = vec![0; rect.2 * rect.3 * 4];

        // Render the walls

        buffer
            .par_rchunks_exact_mut(height as usize * 4)
            .enumerate()
            .for_each(|(x, line)| {

            let ceiling_tile = world.get_ceiling_tile();
            let mut ceiling_is_textured = false;

            let floor_tile = world.get_floor_tile();
            let mut floor_is_textured = false;

            let mut z_buffer = f32::MAX;

            // Ceiling color
            if let Some(ceiling) = ceiling_tile {
                if let Some(color) = ceiling.color {
                    for y in 0.. rect.3 / 2 {
                        let o = y*4;
                        line[o..o+4].copy_from_slice(&color);
                    }
                } else {
                    ceiling_is_textured = true;
                }
            }


            // Floor color
            if let Some(floor) = floor_tile {
                if let Some(color) = floor.color {
                    for y in rect.3 / 2..rect.3 {
                        let o = y*4;
                        line[o..o+4].copy_from_slice(&color);
                    }
                } else {
                    floor_is_textured = true;
                }
            }

            // Texture the ceiling and floor

            if ceiling_is_textured || floor_is_textured {

                for y in rect.3 /2..rect.3 {

                    // rayDir for leftmost ray (x = 0) and rightmost ray (x = w)
                    let ray_dir_x0 = dir.x - plane.x;
                    let ray_dir_y0 = dir.y - plane.y;
                    let ray_dir_x1 = dir.x + plane.x;
                    let ray_dir_y1 = dir.y + plane.y;

                    // Current y position compared to the center of the screen (the horizon)
                    let p = y - rect.3 / 2;

                    // Vertical position of the camera.
                    let pos_z = 0.5 * rect.3 as f32;

                    // Horizontal distance from the camera to the floor for the current row.
                    // 0.5 is the z position exactly in the middle between floor and ceiling.
                    let row_distance = pos_z / p as f32;

                    // calculate the real world step vector we have to add for each x (parallel to camera plane)
                    // adding step by step avoids multiplications with a weight in the inner loop
                    let floor_step_x = row_distance * (ray_dir_x1 - ray_dir_x0) / rect.2 as f32;
                    let floor_step_y = row_distance * (ray_dir_y1 - ray_dir_y0) / rect.2 as f32;

                    // real world coordinates of the leftmost column. This will be updated as we step to the right.
                    let mut floor_x = pos.x + row_distance * ray_dir_x0;
                    let mut floor_y = pos.y + row_distance * ray_dir_y0;

                    floor_x += floor_step_x * x as f32;
                    floor_y += floor_step_y * x as f32;

                    let mix_factor = row_distance / world.fog_distance;

                    // the cell coord is simply got from the integer parts of floorX and floorY
                    let cell_x = floor_x.floor();
                    let cell_y = floor_y.floor();


                    if let Some(floor) = floor_tile {
                        if let Some((image_id, rect)) = floor.texture {
                            let tex_x = ((rect.2 as f32 * (floor_x - cell_x)) as usize).clamp(0, rect.2 - 1);
                            let tex_y = ((rect.3 as f32 * (floor_y - cell_y)) as usize).clamp(0, rect.3 - 1);

                            if let Some((tex_data, tex_width, _tex_height)) = world.get_image(image_id) {
                                let tex_off = rect.0 + tex_x * 4 + rect.1 + ((tex_y as usize) * *tex_width as usize * 4);
                                let off = y * 4;//x * 4 + y * 4 * stride;

                                if mix_factor <= 0.0 {
                                    line[off..off+4].copy_from_slice(&tex_data[tex_off..tex_off+4]);
                                } else
                                if mix_factor >= 1.0 {
                                    line[off..off+4].copy_from_slice(&world.fog_color);
                                } else {
                                    let mut floor_color : [u8;4] = [0, 0, 0, 0];
                                    floor_color.copy_from_slice(&tex_data[tex_off..tex_off+4]);
                                    let color = self.mix_color(&floor_color, &world.fog_color, mix_factor);
                                    line[off..off+4].copy_from_slice(&color);
                                }
                            }
                        }
                    }

                    if let Some(ceiling) = ceiling_tile {
                        if let Some((image_id, tex_rect)) = ceiling.texture {
                            let tex_x = ((tex_rect.2 as f32 * (floor_x - cell_x)) as usize).clamp(0, tex_rect.2 - 1);
                            let tex_y = ((tex_rect.3 as f32 * (floor_y - cell_y)) as usize).clamp(0, tex_rect.3 - 1);

                            if let Some((tex_data, tex_width, _tex_height)) = world.get_image(image_id) {
                                let tex_off = tex_rect.0 + tex_x * 4 + tex_rect.1 + ((tex_y as usize) * *tex_width as usize * 4);
                                let off = (rect.3 - y  - 1) * 4;

                                if mix_factor <= 0.0 {
                                    line[off..off+4].copy_from_slice(&tex_data[tex_off..tex_off+4]);
                                } else
                                if mix_factor >= 1.0 {
                                    line[off..off+4].copy_from_slice(&world.fog_color);
                                } else {
                                    let mut ceiling_color : [u8;4] = [0, 0, 0, 0];
                                    ceiling_color.copy_from_slice(&tex_data[tex_off..tex_off+4]);
                                    let color = self.mix_color(&ceiling_color, &world.fog_color, mix_factor);

                                    line[off..off+4].copy_from_slice(&color);
                                }
                            }
                        }
                    }
                }
            }

            let camera_x = 2.0 * x as f32 / width as f32 - 1.0; //x-coordinate in camera space
            let ray_dir_x = dir.x + plane.x * camera_x;
            let ray_dir_y = dir.y + plane.y * camera_x;

            // which box of the map we're in
            let mut map_x = pos.x as i32;
            let mut map_y = pos.y as i32;

            // length of ray from current position to next x or y-side
            let mut side_dist_x;
            let mut side_dist_y;

            // length of ray from one x or y-side to next x or y-side
            let delta_dist_x = if ray_dir_x == 0.0 { f32::MAX } else { (1.0 / ray_dir_x).abs() };
            let delta_dist_y = if ray_dir_y == 0.0 { f32::MAX } else { (1.0 / ray_dir_y).abs() };
            let perp_wall_dist;

            // what direction to step in x or y-direction (either +1 or -1)
            let step_x;
            let step_y;

            let mut hit = false; //was there a wall hit?
            let mut side = 0; //was a NS or a EW wall hit?

            // calculate step and initial sideDist
            if ray_dir_x < 0.0 {
                step_x = -1;
                side_dist_x = (pos.x - map_x as f32) * delta_dist_x;
            } else {
                step_x = 1;
                side_dist_x = (map_x as f32 + 1.0 - pos.x) * delta_dist_x;
            }

            if ray_dir_y < 0.0 {
                step_y = -1;
                side_dist_y = (pos.y - map_y as f32) * delta_dist_y;
            } else {
                step_y = 1;
                side_dist_y = (map_y as f32 + 1.0 - pos.y) * delta_dist_y;
            }

            // perform DDA
            for _ in 0..40 {
                // jump to next map square, either in x-direction, or in y-direction
                if side_dist_x < side_dist_y {
                    side_dist_x += delta_dist_x;
                    map_x += step_x;
                    side = 0;
                } else {
                    side_dist_y += delta_dist_y;
                    map_y += step_y;
                    side = 1;
                }

                // check if ray has hit a wall
                if world.has_wall(map_x, map_y) == true {
                    hit = true;
                    break;
                }
            }

            if hit {

                // calculate distance projected on camera direction (Euclidean distance would give fisheye effect!)
                if side == 0 {
                    perp_wall_dist = side_dist_x - delta_dist_x;
                } else {
                    perp_wall_dist = side_dist_y - delta_dist_y;
                }

                // calculate height of line to draw on screen
                let line_height = (height as f32 / perp_wall_dist) as i32;

                // calculate lowest and highest pixel to fill in current stripe
                let mut draw_start = -line_height / 2 + height / 2;
                if draw_start < 0 {
                    draw_start = 0;
                }

                let mut draw_end = line_height / 2 + height / 2;
                if draw_end >= height {
                    draw_end = height - 1;
                }

                let mix_factor = perp_wall_dist / world.fog_distance;

                if let Some(tile) = world.get_wall(map_x, map_y) {

                    if let Some((image_id, rect)) = self.get_texture(&tile) {

                        // texturing calculations

                        // calculate value of wall_x
                        let mut wall_x; //where exactly the wall was hit
                        if side == 0 {
                            wall_x = pos.y + perp_wall_dist * ray_dir_y;
                        } else {
                            wall_x = pos.x + perp_wall_dist * ray_dir_x;
                        }
                        wall_x -= wall_x.floor();

                        // x coordinate on the texture
                        let mut tex_x = (wall_x * rect.2 as f32) as usize;
                        if side == 0 && ray_dir_x > 0.0 {
                            tex_x = rect.2 - tex_x - 1;
                        }
                        if side == 1 && ray_dir_y < 0.0 {
                            tex_x = rect.2 - tex_x - 1;
                        }

                        // How much to increase the texture coordinate per screen pixel
                        let step = 1.0 * rect.3 as f32 / line_height as f32;

                        let mut tex_pos = (draw_start - height / 2 + line_height / 2) as f32 * step;

                        if let Some((tex_data, tex_width, _tex_height)) = world.get_image(image_id) {
                            for y in draw_start..draw_end {
                                let tex_off = rect.0 + tex_x * 4 + rect.1 + ((tex_pos as usize) * *tex_width as usize * 4);
                                let off = y as usize * 4;// + x as usize * 4 * stride;

                                if mix_factor <= 0.0 {
                                    line[off..off+4].copy_from_slice(&tex_data[tex_off..tex_off+4]);
                                } else
                                if mix_factor >= 1.0 {
                                    line[off..off+4].copy_from_slice(&world.fog_color);
                                } else {
                                    let mut wall_color : [u8;4] = [0, 0, 0, 0];
                                    wall_color.copy_from_slice(&tex_data[tex_off..tex_off+4]);
                                    let color = self.mix_color(&wall_color, &world.fog_color, mix_factor);

                                    line[off..off+4].copy_from_slice(&color);
                                }

                                tex_pos += step;
                            }
                        }
                        /*
                        // Starting texture coordinate
                        double texPos = (drawStart - h / 2 + lineHeight / 2) * step;
                        for(int y = drawStart; y<drawEnd; y++)
                        {
                            // Cast the texture coordinate to integer, and mask with (texHeight - 1) in case of overflow
                            int texY = (int)texPos & (texHeight - 1);
                            texPos += step;
                            Uint32 color = texture[texNum][texHeight * texY + texX];
                            //make color darker for y-sides: R, G and B byte each divided through two with a "shift" and an "and"
                            if(side == 1) color = (color >> 1) & 8355711;
                            buffer[y][x] = color;
                        }*/
                    }
                }

                // perpendicular distance is stored in the z-buffer for sprite casting
                z_buffer = perp_wall_dist;

                /* color
                let off_x = x * 4;

                for y in draw_start..draw_end {
                    let off = off_x + y as usize * 4 * self.width;
                    frame[off] = 255;
                    frame[off + 1] = 255;
                    frame[off + 2] = 255;
                    frame[off + 3] = 255;
                }
                */
            }

            // Render the sprites

            for sprite in &world.sprites {

                // translate sprite position to relative to camera
                let sprite_x = sprite.x - pos.x;
                let sprite_y = sprite.y - pos.y;

                // transform sprite with the inverse camera matrix
                // [ planeX   dirX ] -1                                       [ dirY      -dirX ]
                // [               ]       =  1/(planeX*dirY-dirX*planeY) *   [                 ]
                // [ planeY   dirY ]                                          [ -planeY  planeX ]

                let inv_det = 1.0 / (plane.x * dir.y - dir.x * plane.y); //required for correct matrix multiplication

                let transform_x = inv_det * (dir.y * sprite_x - dir.x * sprite_y);
                let transform_y = inv_det * (-plane.y * sprite_x + plane.x * sprite_y); //this is actually the depth inside the screen, that what Z is in 3D

                let mix_factor = transform_y / world.fog_distance;

                let sprite_screen_x = ((width as f32 / 2.0) * (1.0 + transform_x / transform_y)) as i32;

                // calculate height of the sprite on screen
                let sprite_height = ((height as f32 / (transform_y)) as i32).abs(); //using 'transformY' instead of the real distance prevents fisheye
                // calculate lowest and highest pixel to fill in current stripe
                let mut draw_start_y = -sprite_height / 2 + height / 2;
                if draw_start_y < 0 { draw_start_y = 0; }
                let mut draw_end_y = sprite_height / 2 + height / 2;
                if draw_end_y >= height { draw_end_y = height - 1; }

                // calculate width of the sprite
                let sprite_width = ((height as f32 / (transform_y)) as i32).abs();
                let mut draw_start_x = -sprite_width / 2 + sprite_screen_x;
                if draw_start_x < 0 { draw_start_x = 0; }
                let mut draw_end_x = sprite_width / 2 + sprite_screen_x;
                if draw_end_x >= width { draw_end_x = width - 1; }

                if let Some((image_id, tex_rect)) = self.get_texture(&sprite.tile) {
                    if let Some((tex_data, tex_width, _tex_height)) = world.get_image(image_id) {

                        // loop through every vertical stripe of the sprite on screen
                        let stripe = x as i32;
                        if stripe >= draw_start_x && stripe < draw_end_x {
                        //for stripe in draw_start_x..draw_end_x {
                            let tex_x = ((256 * (stripe - (-sprite_width / 2 + sprite_screen_x)) * tex_rect.2 as i32 / sprite_width) / 256) as usize;

                            // the conditions in the if are:
                            // 1) it's in front of camera plane so you don't see things behind you
                            // 2) it's on the screen (left)
                            // 3) it's on the screen (right)
                            // 4) ZBuffer, with perpendicular distance

                            if transform_y > 0.0 && stripe > 0 && stripe < width && transform_y < z_buffer {
                                for y in draw_start_y as usize .. draw_end_y as usize {

                                    let d = (y) * 256 - height as usize * 128 + sprite_height as usize * 128; //256 and 128 factors to avoid floats
                                    let tex_y = (((d * tex_rect.3) / sprite_height as usize) / 256) as usize;

                                    let tex_off = tex_rect.0 + tex_x * 4 + tex_rect.1 + ((tex_y as usize) * *tex_width as usize * 4);
                                    let off = y * 4;//(rect.0 + stripe as usize) * 4 + (rect.1 + y as usize) * 4 * stride;

                                    if mix_factor <= 0.0 {
                                        line[off..off+4].copy_from_slice(&tex_data[tex_off..tex_off+4]);
                                    } else
                                    if mix_factor >= 1.0 {
                                        // line[off..off+4].copy_from_slice(&world.fog_color);
                                    } else {
                                        let mut wall_color : [u8;4] = [0, 0, 0, 0];
                                        wall_color.copy_from_slice(&tex_data[tex_off..tex_off+4]);
                                        let tex_alpha = tex_data[tex_off+3] as f32 / 255.0;
                                        if tex_alpha > 0.0 {
                                            let color = self.mix_color(&wall_color, &world.fog_color, mix_factor * tex_alpha);
                                            line[off..off+4].copy_from_slice(&color);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        // Copy the buffer into the frame 90 degrees rotated
        frame
            .par_rchunks_exact_mut(in_stride as usize * 4)
            .enumerate()
            .for_each(|(y, line)| {
                for x in 0..in_rect.2 {
                    let off = (in_rect.0 + x) * 4;
                    let buffer_off = (rect.3 - y - 1) * 4 + (rect.2 - x - 1) * 4 * stride;
                    line[off..off+4].copy_from_slice(&buffer[buffer_off..buffer_off+4]);
                }
            });

        let stop = self.get_time();
        println!("tick time {:?}", stop - start);

        self.old_time = self.time;
        self.time = self.get_time();

        if self.old_time > 0 {
            self.anim_curr_time += self.time - self.old_time;
        }

        let frame_time = (self.time - self.old_time) as f32 / 1000.0;
        // println!("fps {}", 1.0 / frame_time); //FPS counter

        self.move_speed = frame_time * 5.0; //the constant value is in squares/second
        self.rot_speed = frame_time * 2.0;
    }

    /// Go forward
    pub fn go_forward(&mut self, world: &WorldMap) {
        if world.has_wall((self.pos.x + self.dir.x * self.move_speed) as i32, self.pos.y as i32) == false {
            self.pos.x += self.dir.x * self.move_speed;
        }

        if world.has_wall(self.pos.x as i32, (self.pos.y + self.dir.y * self.move_speed) as i32) == false {
            self.pos.y += self.dir.y * self.move_speed;
        }
    }

    /// Go backward
    pub fn go_backward(&mut self, world: &WorldMap) {
        if world.has_wall((self.pos.x - self.dir.x * self.move_speed) as i32, self.pos.y as i32) == false {
            self.pos.x -= self.dir.x * self.move_speed;
        }

        if world.has_wall(self.pos.x as i32, (self.pos.y - self.dir.y * self.move_speed) as i32) == false {
            self.pos.y -= self.dir.y * self.move_speed;
        }
    }

    /// Turn left
    pub fn turn_left(&mut self) {
        let old_dir_x = self.dir.x;
        self.dir.x = self.dir.x * self.rot_speed.cos() - self.dir.y * self.rot_speed.sin();
        self.dir.y = old_dir_x * self.rot_speed.sin() + self.dir.y * self.rot_speed.cos();

        let old_plane_x = self.plane.x;
        self.plane.x = self.plane.x * self.rot_speed.cos() - self.plane.y * self.rot_speed.sin();
        self.plane.y = old_plane_x * self.rot_speed.sin() + self.plane.y * self.rot_speed.cos();
    }

    /// Turn right
    pub fn turn_right(&mut self) {
        let old_dir_x = self.dir.x;
        self.dir.x = self.dir.x * (-self.rot_speed).cos() - self.dir.y * (-self.rot_speed).sin();
        self.dir.y = old_dir_x * (-self.rot_speed).sin() + self.dir.y * (-self.rot_speed).cos();

        let old_plane_x = self.plane.x;
        self.plane.x = self.plane.x * (-self.rot_speed).cos() - self.plane.y * (-self.rot_speed).sin();
        self.plane.y = old_plane_x * (-self.rot_speed).sin() + self.plane.y * (-self.rot_speed).cos();
    }

    /// Set the position
    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.pos.x = x as f32;
        self.pos.y = y as f32;
    }

    /// Set the animation time in ms
    pub fn set_anim_time(&mut self, time: u16) {
        self.anim_time = time as u128;
    }

    /// Gets the current time in milliseconds
    fn get_time(&self) -> u128 {
        let stop = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
            stop.as_millis()
    }

    #[inline(always)]
    /// Mix two colors
    fn mix_color(&self, a: &[u8;4], b: &[u8;4], v: f32) -> [u8; 4] {
        [   (((1.0 - v) * (a[0] as f32 / 255.0) + b[0] as f32 / 255.0 * v) * 255.0) as u8,
            (((1.0 - v) * (a[1] as f32 / 255.0) + b[1] as f32 / 255.0 * v) * 255.0) as u8,
            (((1.0 - v) * (a[2] as f32 / 255.0) + b[2] as f32 / 255.0 * v) * 255.0) as u8,
        255]
    }

    #[inline(always)]
    /// Returns the tile rect for a given texture, handles animation
    fn get_texture(&self, tile: &Tile) -> Option<(usize, (usize, usize, usize, usize))> {
        if let Some((image_id, rect)) = tile.texture {
            if tile.frames == 1 {
                return Some((image_id, rect));
            } else {
                let frame = self.anim_counter % tile.frames as usize;
                let x = rect.0 + (rect.2 * frame as usize * 4);
                let y = rect.1;
                return Some((image_id, (x, y, rect.2, rect.3)));
            }
        }
        None
    }

}