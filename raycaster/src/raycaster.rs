use crate::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Raycaster {
    world                   : WorldMap,

    width                   : usize,
    height                  : usize,

    time                    : u128,
    old_time                : u128,

    move_speed              : f32,
    rot_speed               : f32,

    pos                     : vec2,
    dir                     : vec2,
    plane                   : vec2,
}

impl Raycaster {

    pub fn new(map: WorldMap, width: usize, height: usize) -> Self {

        Self {
            world           : map,

            width           : width,
            height          : height,

            pos             : vec2::new(0.0, 5.0),
            dir             : vec2::new(-1.0, 0.0),
            plane           : vec2::new(0.0, 0.66),

            time            : 0,
            old_time        : 0,

            move_speed      : 0.0,
            rot_speed       : 0.0,
        }
    }

    pub fn render(&mut self, frame: &mut [u8]) {

        //let start = self.get_time();

        let width = self.width as i32;
        let height = self.height as i32;

        let pos = self.pos.clone();
        let dir = self.dir.clone();
        let plane = self.plane.clone();

        frame.fill(0);

        for x in 0..self.width {

            let camera_x = 2.0 * x as f32 / width as f32 - 1.0; //x-coordinate in camera space
            let ray_dir_x = dir.x + plane.x * camera_x;
            let ray_dir_y = dir.y + plane.y * camera_x;

            //which box of the map we're in
            let mut map_x = pos.x as i32;
            let mut map_y = pos.y as i32;

            //length of ray from current position to next x or y-side
            let mut side_dist_x;
            let mut side_dist_y;

            //length of ray from one x or y-side to next x or y-side
            let delta_dist_x = if ray_dir_x == 0.0 { f32::MAX } else { (1.0 / ray_dir_x).abs() };
            let delta_dist_y = if ray_dir_y == 0.0 { f32::MAX } else { (1.0 / ray_dir_y).abs() };
            let perp_wall_dist;

            //what direction to step in x or y-direction (either +1 or -1)
            let step_x;
            let step_y;

            let mut hit = false; //was there a wall hit?
            let mut side = 0; //was a NS or a EW wall hit?

            //calculate step and initial sideDist
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

            //perform DDA
            for _ in 0..20 {
                //jump to next map square, either in x-direction, or in y-direction
                if side_dist_x < side_dist_y {
                    side_dist_x += delta_dist_x;
                    map_x += step_x;
                    side = 0;
                } else {
                    side_dist_y += delta_dist_y;
                    map_y += step_y;
                    side = 1;
                }
                //Check if ray has hit a wall
                if self.world.has_wall(map_x, map_y) == true {
                    hit = true;
                    break;
                }
            }

            if hit {

                //println!("x: {}. Hit at {}, {}", x, map_x, map_y);

                //Calculate distance projected on camera direction (Euclidean distance would give fisheye effect!)
                if side == 0 {
                    perp_wall_dist = side_dist_x - delta_dist_x;
                } else {
                    perp_wall_dist = side_dist_y - delta_dist_y;
                }

                //Calculate height of line to draw on screen
                let line_height = (height as f32 / perp_wall_dist) as i32;

                //calculate lowest and highest pixel to fill in current stripe
                let mut draw_start = -line_height / 2 + height / 2;
                if draw_start < 0 {
                    draw_start = 0;
                }

                let mut draw_end = line_height / 2 + height / 2;
                if draw_end >= height {
                    draw_end = height - 1;
                }

                //println!("x: {}, draw_start: {}, draw_end: {}", x, draw_start, draw_end);

                let off_x = x * 4;

                for y in draw_start..draw_end {
                    let off = off_x + y as usize * 4 * self.width;
                    frame[off] = 255;
                    frame[off + 1] = 255;
                    frame[off + 2] = 255;
                    frame[off + 3] = 255;
                }
            }
        }

        //let stop = self.get_time();
        //println!("tick time {:?}", stop - start);

        self.old_time = self.time;
        self.time = self.get_time();

        let frame_time = (self.time - self.old_time) as f32 / 1000.0;
        // println!("fps {}", 1.0 / frame_time); //FPS counter

        self.move_speed = frame_time * 5.0; //the constant value is in squares/second
        self.rot_speed = frame_time * 3.0;
    }

    /// The window has been resized
    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    /// Go forward
    pub fn go_forward(&mut self) {
        if self.world.has_wall((self.pos.x + self.dir.x * self.move_speed) as i32, self.pos.y as i32) == false {
            self.pos.x += self.dir.x * self.move_speed;
        }

        if self.world.has_wall(self.pos.x as i32, (self.pos.y + self.dir.y * self.move_speed) as i32) == false {
            self.pos.y += self.dir.y * self.move_speed;
        }
    }

    /// Go backward
    pub fn go_backward(&mut self) {
        if self.world.has_wall((self.pos.x - self.dir.x * self.move_speed) as i32, self.pos.y as i32) == false {
            self.pos.x -= self.dir.x * self.move_speed;
        }

        if self.world.has_wall(self.pos.x as i32, (self.pos.y - self.dir.y * self.move_speed) as i32) == false {
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

    /// Gets the current time in milliseconds
    fn get_time(&self) -> u128 {
        let stop = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
            stop.as_millis()
    }

}