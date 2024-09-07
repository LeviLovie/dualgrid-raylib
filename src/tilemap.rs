use log::{info, error};
use raylib::prelude::*;

pub struct Chunk {
    pub x: i32,
    pub y: i32,
    pub size_x: i32,
    pub size_y: i32,
    pub data: Vec<Vec<bool>>,
}

impl Chunk {
    pub fn new(x: i32, y: i32, size_x: i32, size_y: i32, data: Vec<Vec<bool>>) -> Self {
        info!("Chunk created at ({}, {}) with size ({}, {})", x, y, size_x, size_y);
        Self { x, y, size_x, size_y, data }
    }

    pub fn get(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.size_x || y < 0 || y >= self.size_y {
            return false;
        }

        self.data[y as usize][x as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, value: bool) {
        if x < 0 || x >= self.size_x || y < 0 || y >= self.size_y {
            return;
        }

        self.data[y as usize][x as usize] = value;
    }
}

pub struct TileRule {
    pub neighbors: [bool; 4], // Left Top, Right Top, Right Bottom, Left Bottom
    pub sprite: Texture2D,
    pub size: i32,
}

pub struct TileRules {
    pub rules: Vec<TileRule>,
}

impl TileRules {
    pub fn new(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        yaml_file: &str,
        sprite_atlas: &str,
    ) -> Self {
        let f = match std::fs::File::open(yaml_file) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to open the {} file: {}", yaml_file, e);
                std::process::exit(1);
            }
        };

        let data: serde_yaml::Value = match serde_yaml::from_reader(f) {
            Ok(d) => d,
            Err(e) => {
                error!("Failed to parse the {} file: {}", yaml_file, e);
                std::process::exit(1);
            }
        };

        // Yaml:
        // size: 16
        //
        // rules:
        //   - neighbors: [0, 0, true, false]
        //     sprite: { x: 0, y: 0 }
        //   ...
        //   - neighbors: [true, 0, 0, false]
        //     sprite: { x: 48, y: 48 }

        let size = match data["size"].as_i64() {
            Some(size) => size as i32,
            None => {
                error!("Invalid size value");
                std::process::exit(1);
            }
        };

        let rules: Vec<TileRule> = match data["rules"].as_sequence() {
            Some(rules) => rules
                .iter()
                .map(|rule| {
                    let neighbors = match rule["neighbors"].as_sequence() {
                        Some(neighbors) => {
                            let mut n = [false; 4];
                            for (i, neighbor) in neighbors.iter().enumerate() {
                                n[i] = match neighbor.as_bool() {
                                    Some(b) => b,
                                    None => {
                                        error!("Invalid neighbor value");
                                        std::process::exit(1);
                                    }
                                };
                            }
                            n
                        }
                        None => {
                            error!("Invalid neighbors value");
                            std::process::exit(1);
                        }
                    };

                    let sprite_rect = match rule["sprite"].as_mapping() {
                        Some(sprite) => {
                            let x = match sprite.get(&serde_yaml::Value::String("x".to_string())) {
                                Some(x) => match x.as_i64() {
                                    Some(x) => x as f32,
                                    None => {
                                        error!("Invalid x value");
                                        std::process::exit(1);
                                    }
                                },
                                None => {
                                    error!("Invalid x value");
                                    std::process::exit(1);
                                }
                            };

                            let y = match sprite.get(&serde_yaml::Value::String("y".to_string())) {
                                Some(y) => match y.as_i64() {
                                    Some(y) => y as f32,
                                    None => {
                                        error!("Invalid y value");
                                        std::process::exit(1);
                                    }
                                },
                                None => {
                                    error!("Invalid y value");
                                    std::process::exit(1);
                                }
                            };

                            Rectangle::new(x, y, size as f32, size as f32)
                        }
                        None => {
                            error!("Invalid sprite value");
                            std::process::exit(1);
                        }
                    };

                    // Load the sprite as an image, crop it and convert it to a texture
                    let mut image = match Image::load_image(&sprite_atlas) {
                        Ok(image) => image,
                        Err(e) => {
                            error!("Failed to load the sprite atlas image: {}", e);
                            std::process::exit(1);
                        }
                    };
                    image.crop(sprite_rect);
                    let texture = rl.load_texture_from_image(&thread, &image).unwrap();

                    TileRule {
                        neighbors,
                        sprite: texture,
                        size,
                    }
                })
                .collect(),
            None => {
                error!("Invalid rules value");
                std::process::exit(1);
            }
        };

        TileRules { rules }
    }

    pub fn tile_by_rules(&self, neighbors: [bool; 4]) -> &TileRule {
        match self.rules.iter().find(|rule| rule.neighbors == neighbors) {
            Some(rule) => &rule,
            None => {
                error!("Neighbors value not found in the rules");
                std::process::exit(1);
            }
        }
    }
}

pub struct TileMap {
    pub rules: TileRules,
    pub chunks: Vec<Chunk>,
}

impl TileMap {
    pub fn new(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        yaml_file: &str,
        sprite_atlas: &str,
    ) -> Self {
        let rules = TileRules::new(rl, thread, yaml_file, sprite_atlas);
        let chunks = vec![];

        Self { rules, chunks }
    }

    pub fn load_chunk_from_yaml(&mut self, yaml_file: &str) {
        let f = match std::fs::File::open(yaml_file) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to open the {} file: {}", yaml_file, e);
                std::process::exit(1);
            }
        };

        let data: serde_yaml::Value = match serde_yaml::from_reader(f) {
            Ok(d) => d,
            Err(e) => {
                error!("Failed to parse the {} file: {}", yaml_file, e);
                std::process::exit(1);
            }
        };

        // Yaml:
        // x: 0
        // y: 0
        // size: 2
        // data:
        //   - [0, 0]
        //   - [0, 0]

        let x = match data["x"].as_i64() {
            Some(x) => x as i32,
            None => {
                error!("Invalid x value in the chunk data");
                std::process::exit(1);
            }
        };

        let y = match data["y"].as_i64() {
            Some(y) => y as i32,
            None => {
                error!("Invalid y value in the chunk data");
                std::process::exit(1);
            }
        };

        let size_x = match data["size_x"].as_i64() {
            Some(size) => size as i32,
            None => {
                error!("Invalid size x value in the chunk data");
                std::process::exit(1);
            }
        };

        let size_y = match data["size_y"].as_i64() {
            Some(size) => size as i32,
            None => {
                error!("Invalid size y value in the chunk data");
                std::process::exit(1);
            }
        };

        let chunk = Chunk::new(
            x,
            y,
            size_x,
            size_y,
            match data["data"].as_sequence() {
                Some(data) => data
                    .iter()
                    .map(|row| match row.as_sequence() {
                        Some(row) => row
                            .iter()
                            .map(|value| match value.as_u64() {
                                Some(value) => value == 1,
                                None => {
                                    error!("Invalid value in the chunk data");
                                    std::process::exit(1);
                                }
                            })
                            .collect(),
                        None => {
                            error!("Invalid row value in the chunk data");
                            std::process::exit(1);
                        }
                    })
                    .collect(),
                None => {
                    error!("Invalid data value in the chunk data");
                    std::process::exit(1);
                }
            },
        );

        self.chunks.push(chunk);
    }

    pub fn get(&self, x: i32, y: i32) -> bool {
        for chunk in self.chunks.iter() {
            if x >= chunk.x && x < chunk.x + chunk.size_x && y >= chunk.y && y < chunk.y + chunk.size_y {
                return chunk.get(x - chunk.x, y - chunk.y);
            }
        }

        return false;
    }

    pub fn set(&mut self, x: i32, y: i32, value: bool) {
        for chunk in self.chunks.iter_mut() {
            if x >= chunk.x && x < chunk.x + chunk.size_x && y >= chunk.y && y < chunk.y + chunk.size_y {
                chunk.set(x - chunk.x, y - chunk.y, value);
                return;
            }
        }
    }

    pub fn add_chunk(&mut self, x: i32, y: i32, size_x: i32, size_y: i32) {
        let chunk = Chunk::new(x, y, size_x, size_y, vec![vec![false; size_x as usize]; size_y as usize]);
        self.chunks.push(chunk);
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        for chunk in self.chunks.iter() {
            // -1 Cause we want to draw the left and top edge tiles not present in any chunks
            for y in -1..chunk.size_y {
                for x in -1..chunk.size_x {
                    let neighbors = [
                        chunk.get(x, y),
                        self.get(x + 1 + chunk.x, y + chunk.y),
                        self.get(x + chunk.x, y + 1 + chunk.y),
                        self.get(x + 1 + chunk.x, y + 1 + chunk.y),
                    ];

                    let sprite_rule = self.rules.tile_by_rules(neighbors);

                    d.draw_texture_pro(
                        &sprite_rule.sprite,
                        Rectangle::new(0.0, 0.0, sprite_rule.size as f32, sprite_rule.size as f32),
                        Rectangle::new(
                            (chunk.x + x) as f32 * sprite_rule.size as f32 * 4.0
                                    + sprite_rule.size as f32 * 4.0 / 2.0,
                            (chunk.y + y) as f32 * sprite_rule.size as f32 * 4.0
                                    + sprite_rule.size as f32 * 4.0 / 2.0,
                            sprite_rule.size as f32 * 4.0,
                            sprite_rule.size as f32 * 4.0,
                        ),
                        Vector2::new(0.0, 0.0),
                        0.0,
                        Color::WHITE,
                    );
                }
            }
        }
    }
}
