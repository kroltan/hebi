use crate::{
    config::{Cell, MapData, MapType},
    Direction,
};

use rand::prelude::*;
use rand_pcg::Pcg64;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(default)]
pub struct CorridorsMap {
    pub width: u32,
    pub height: u32,
    corridor_width: u32,
    corridor_height: u32,
    top_corridor_offset: i32,
    bottom_corridor_offset: i32,
    wall_variance: f32,
}

impl Default for CorridorsMap {
    fn default() -> Self {
        Self {
            width: 34,
            height: 17,
            corridor_width: 3,
            corridor_height: 10,
            top_corridor_offset: 3,
            bottom_corridor_offset: 0,
            wall_variance: 0.5,
        }
    }
}

impl MapType for CorridorsMap {
    fn get_map_data(&self, generator: &mut rand_pcg::Pcg64) -> MapData {
        let width = self.width;
        let height = self.height;
        let corridor_width = self.corridor_width;
        let corridor_height = self.corridor_height;
        let top_corridor_offset = self.top_corridor_offset;
        let bottom_corridor_offset = self.bottom_corridor_offset;
        let wall_variance = self.wall_variance;
        MapData {
            width,
            height,
            cells: {
                let mut cells = HashMap::new();
                let corridor_width = corridor_width as u32;
                let mut top_wall_heights = HashMap::<u32, u32>::new();
                let mut bottom_wall_heights = HashMap::<u32, u32>::new();
                let get_wall_height =
                    |hash_map: &mut HashMap<u32, u32>, generator: &mut Pcg64, x: u32| {
                        *hash_map.entry(x).or_insert_with(|| {
                            let corridor_height = corridor_height as f32;
                            (corridor_height * (1.0 - wall_variance)
                                + corridor_height * wall_variance * generator.gen::<f32>())
                                as u32
                        })
                    };
                let mut gap = 1;
                let mut blocked = false;
                for x in 0..width {
                    let previously_blocked = blocked;
                    blocked = true;
                    for y in 0..height {
                        cells.insert((x, y), {
                            if x == 0
                                || x == width - 1
                                || y == 0
                                || y == height - 1
                                || ((x as i32 - top_corridor_offset) % (corridor_width as i32 + 1)
                                    == 0
                                    && x > 2
                                    && x < width - corridor_width - 1
                                    && (y as i32)
                                        < get_wall_height(&mut top_wall_heights, generator, x)
                                            as i32
                                            + 1)
                                || ((x as i32 - bottom_corridor_offset)
                                    % (corridor_width as i32 + 1)
                                    == 0
                                    && x > 2
                                    && x < width - corridor_width - 1
                                    && y as i32
                                        > (height as i32
                                            - get_wall_height(
                                                &mut bottom_wall_heights,
                                                generator,
                                                x,
                                            ) as i32
                                            - 2))
                            {
                                Cell::Wall
                            } else {
                                blocked = false;
                                Cell::Empty
                            }
                        });
                    }
                    // Check for blocked columns
                    if blocked && x > 0 && x < width - 1 {
                        if !previously_blocked {
                            gap = generator.gen_range(1..(height - 1));
                        }
                        cells.insert((x, gap), Cell::Empty);
                    }
                }
                for (x, wall_height) in bottom_wall_heights.iter() {
                    let x = *x as u32;
                    let y = (height as i32 - *wall_height as i32 - 2) as u32;
                    if y == 0 || y == height - 1 {
                        continue;
                    }
                    cells.insert((x - 1, y), Cell::Empty);
                    cells.insert((x, y), Cell::Empty);
                    cells.insert((x + 1, y), Cell::Empty);
                    cells.insert((x + 1, height - 3), Cell::Spawn(Direction::Up));
                }
                for (x, wall_height) in top_wall_heights.iter() {
                    let x = *x as u32;
                    let y = (wall_height + 1) as u32;
                    if y == 0 || y == height - 1 {
                        continue;
                    }
                    cells.insert((x - 1, y), Cell::Empty);
                    cells.insert((x, y), Cell::Empty);
                    cells.insert((x + 1, y), Cell::Empty);
                    cells.insert((x + 1, 2), Cell::Spawn(Direction::Down));
                }
                cells
            },
        }
    }
}
