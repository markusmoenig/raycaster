use crate::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Raycaster {
    time                    : u128,
    old_time                : u128,

    move_speed              : f32,
    rot_speed               : f32,

    pos                     : vec2,
    dir                     : vec2,
    plane                   : vec2,
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
        }
    }

    /// Renders the world map into the frame inside the given rectangle
    pub fn render(&mut self, frame: &mut [u8], rect: (usize, usize, usize, usize), stride: usize, world: &WorldMap) {

        //let start = self.get_time();

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

        if ceiling_is_textured && floor_is_textured {

            for y in rect.1..rect.3 {

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

                for x in rect.0..rect.2 {

                    // the cell coord is simply got from the integer parts of floorX and floorY
                    let cell_x = floor_x.floor();
                    let cell_y = floor_y.floor();

                    if y >= rect.1 + rect.3 / 2 {
                        if let Some(floor) = floor_tile {
                            if let Some((image_id, rect)) = floor.texture {
                                let tex_x = ((rect.2 as f32 * (floor_x - cell_x)) as usize).clamp(0, rect.2 - 1);
                                let tex_y = ((rect.3 as f32 * (floor_y - cell_y)) as usize).clamp(0, rect.3 - 1);

                                if let Some((tex_data, tex_width, _tex_height)) = world.get_image(image_id) {

                                    let tex_off = rect.0 + tex_x * 4 + rect.1 + ((tex_y as usize) * *tex_width as usize * 4);

                                    let off = x * 4 + y * 4 * stride;
                                    frame[off] = tex_data[tex_off];
                                    frame[off + 1] = tex_data[tex_off + 1];
                                    frame[off + 2] = tex_data[tex_off + 2];
                                    frame[off + 3] = tex_data[tex_off + 3];
                                }

                                floor_x += floor_step_x;
                                floor_y += floor_step_y;
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
                                frame[off] = tex_data[tex_off];
                                frame[off + 1] = tex_data[tex_off + 1];
                                frame[off + 2] = tex_data[tex_off + 2];
                                frame[off + 3] = tex_data[tex_off + 3];
                            }
                        }
                    }

                    /*

                    // get the texture coordinate from the fractional part
                    let tx = (int)(texWidth * (floorX - cellX)) & (texWidth - 1);
                    let ty = (int)(texHeight * (floorY - cellY)) & (texHeight - 1);

                    floorX += floorStepX;
                    floorY += floorStepY;

                    // choose texture and draw the pixel
                    int floorTexture = 3;
                    int ceilingTexture = 6;
                    Uint32 color;

                    // floor
                    color = texture[floorTexture][texWidth * ty + tx];
                    color = (color >> 1) & 8355711; // make a bit darker
                    buffer[y][x] = color;

                    //ceiling (symmetrical, at screenHeight - y - 1 instead of y)
                    color = texture[ceilingTexture][texWidth * ty + tx];
                    color = (color >> 1) & 8355711; // make a bit darker
                    buffer[screenHeight - y - 1][x] = color;
                    */
                }
            }
        }


        // Render the walls

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

                if let Some(tile) = world.get_wall(map_x, map_y) {

                    if let Some((image_id, rect)) = tile.texture {

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
                                frame[off] = tex_data[tex_off];
                                frame[off + 1] = tex_data[tex_off + 1];
                                frame[off + 2] = tex_data[tex_off + 2];
                                frame[off + 3] = tex_data[tex_off + 3];

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

        //let stop = self.get_time();
        //println!("tick time {:?}", stop - start);

        self.old_time = self.time;
        self.time = self.get_time();

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

    /// Gets the current time in milliseconds
    fn get_time(&self) -> u128 {
        let stop = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
            stop.as_millis()
    }

}