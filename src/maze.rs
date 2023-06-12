use base64::{engine::general_purpose, Engine};
use maze_generator::ellers_algorithm::EllersGenerator;
use maze_generator::prelude::*;
use parking_lot::Mutex;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

use crate::{
    compile::MAP_NAME,
    mesh::{mesh_to_brush, Mesh},
    PLUGIN,
};

const SIZE_SEGMENT: f32 = 256.;
const CEILING_TEXTURE: &str = "maze/tiles";
const FLOOR_TEXTURE: &str = "maze/floor";
const WALL_TEXTURE: &str = "maze/wallpaper";
const MAZE_X_SIZE: i32 = 30;
const MAZE_Y_SIZE: i32 = MAZE_X_SIZE;
const WALL_THICKNESS: f32 = 5.;

pub static MAP_FILE_BASE_64: Mutex<Vec<String>> = Mutex::new(Vec::new());
pub static LAST_INFO: Mutex<MazeCreationInfo> = Mutex::new(MazeCreationInfo { seed: None });

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MazeCreationInfo {
    seed: Option<[u8; 32]>,
}

impl Default for MazeCreationInfo {
    fn default() -> Self {
        let mut rng = rand::thread_rng();

        let seed: [u8; 32] = rng.gen();

        log::info!("generated random seed : {seed:?}");

        Self { seed: Some(seed) }
    }
}

pub fn create_maze(info: Option<MazeCreationInfo>) {
    let maze_info = info.unwrap_or_default();

    let mut last_info = LAST_INFO.lock();

    let path = PLUGIN
        .wait()
        .mod_path
        .lock()
        .join(format!("Titanfall2/maps/{MAP_NAME}.map"));

    _ = fs::remove_file(&path);
    {
        let mut file = fs::File::create(&path).unwrap();

        writeln!(file, "{{").unwrap();
        writeln!(file, "\"classname\" \"worldspawn\"").unwrap();

        _ = generate_maze(&mut file, &maze_info);

        writeln!(file, "}}").unwrap();

        // temp
        writeln!(
            file,
            r#"
{{
"classname" "info_player_start"
"origin" "128 128 50"
}}
{{
"classname" "prop_dynamic"
"origin" "{} {} 0"
"model" "models/humans/heroes/mlt_hero_sarah.mdl"
"angles" "0 0 0"
"modelscale" "2"
}}"#,
            (MAZE_X_SIZE as f32 * SIZE_SEGMENT) / 2.,
            (MAZE_Y_SIZE as f32 * SIZE_SEGMENT) / 2.
        )
        .unwrap();
    }

    log::info!("map fully written");

    let mut data = MAP_FILE_BASE_64.lock();
    data.clear();

    const SLICE_LENGHT: usize = 100;

    let bin = bincode::serialize(&*last_info).unwrap();

    let mut buf = String::new();
    general_purpose::STANDARD_NO_PAD.encode_string(bin, &mut buf);

    let max_slices = buf.len().div_ceil(SLICE_LENGHT);

    let mut map_file_slices: Vec<String> = if max_slices != 1 {
        (1..max_slices)
            .map(|index| {
                if index >= max_slices.saturating_sub(1) {
                    buf[index.saturating_sub(1) * SLICE_LENGHT..].to_string()
                } else {
                    buf[index.saturating_sub(1) * SLICE_LENGHT..index * SLICE_LENGHT].to_string()
                }
            })
            .collect()
    } else {
        vec![buf]
    };

    data.append(&mut map_file_slices);

    *last_info = maze_info;
}

pub fn generate_maze(file: &mut fs::File, maze_info: &MazeCreationInfo) -> Maze {
    let mut generator = EllersGenerator::new(maze_info.seed);
    let maze = generator.generate(MAZE_X_SIZE, MAZE_Y_SIZE).unwrap();

    log::info!("maze generated");

    (0..maze.size.1)
        .map(|iy| (iy, generate_walls_top(iy, &maze)))
        .flat_map(|(iy, meshes)| combine_vec(meshes, generate_walls_left(iy, &maze)))
        .for_each(|mesh| writeln!(file, "{}", mesh).unwrap());

    log::info!("maze walls created");

    // bottom border
    let x1 = 0.;
    let y1 = (maze.size.1) as f32 * SIZE_SEGMENT;
    let x2 = (maze.size.0) as f32 * SIZE_SEGMENT;
    let y2 = (maze.size.1) as f32 * SIZE_SEGMENT;

    let mesh = mesh_to_brush(
        (x1, y1 + WALL_THICKNESS, 0.0).into(),
        (x2, y2 - WALL_THICKNESS, SIZE_SEGMENT).into(),
        WALL_TEXTURE.to_string(),
    );

    writeln!(file, "{}", mesh).unwrap();

    // right border
    let x1 = (maze.size.0) as f32 * SIZE_SEGMENT;
    let y1 = 0.;
    let x2 = (maze.size.0) as f32 * SIZE_SEGMENT;
    let y2 = (maze.size.1) as f32 * SIZE_SEGMENT;

    let mesh = mesh_to_brush(
        (x1 + WALL_THICKNESS, y1, 0.0).into(),
        (x2 - WALL_THICKNESS, y2, SIZE_SEGMENT).into(),
        WALL_TEXTURE.to_string(),
    );

    writeln!(file, "{}", mesh).unwrap();

    // ceiling
    let x1 = 0.;
    let y1 = 0.;
    let x2 = (maze.size.0) as f32 * SIZE_SEGMENT;
    let y2 = (maze.size.1) as f32 * SIZE_SEGMENT;

    let mesh = mesh_to_brush(
        (x1, y1, SIZE_SEGMENT + WALL_THICKNESS).into(),
        (x2, y2, SIZE_SEGMENT - WALL_THICKNESS).into(),
        CEILING_TEXTURE.to_string(),
    );

    writeln!(file, "{}", mesh).unwrap();

    // floor
    let x1 = 0.;
    let y1 = 0.;
    let x2 = (maze.size.0) as f32 * SIZE_SEGMENT;
    let y2 = (maze.size.1) as f32 * SIZE_SEGMENT;

    let mesh = mesh_to_brush(
        (x1, y1, 0.).into(),
        (x2, y2, -WALL_THICKNESS).into(),
        FLOOR_TEXTURE.to_string(),
    );

    writeln!(file, "{}", mesh).unwrap();

    log::info!("maze fully complete");

    maze
}

fn combine_vec(mut vec1: Vec<Mesh>, mut vec2: Vec<Mesh>) -> Vec<Mesh> {
    vec1.append(&mut vec2);
    vec1
}

fn generate_walls_top(iy: i32, maze: &Maze) -> Vec<Mesh> {
    (0..maze.size.0)
        .filter(|ix| {
            !maze
                .get_field(&(*ix, iy).into())
                .unwrap()
                .has_passage(&Direction::North)
        })
        .map(|ix| generate_wall_top(ix as f32, iy as f32))
        .collect()
}

fn generate_wall_top(ix: f32, iy: f32) -> Mesh {
    let x1 = ix * SIZE_SEGMENT;
    let y1 = iy * SIZE_SEGMENT;
    let x2 = (ix + 1.) * SIZE_SEGMENT;
    let y2 = iy * SIZE_SEGMENT;

    mesh_to_brush(
        (x1, y1 + WALL_THICKNESS, 0.0).into(),
        (x2, y2 - WALL_THICKNESS, SIZE_SEGMENT).into(),
        WALL_TEXTURE.to_string(),
    )
}

fn generate_walls_left(iy: i32, maze: &Maze) -> Vec<Mesh> {
    (0..maze.size.0)
        .filter(|ix| {
            !maze
                .get_field(&(*ix, iy).into())
                .unwrap()
                .has_passage(&Direction::West)
        })
        .map(|ix| generate_wall_left(ix as f32, iy as f32))
        .collect()
}

fn generate_wall_left(ix: f32, iy: f32) -> Mesh {
    let x1 = ix * SIZE_SEGMENT;
    let y1 = iy * SIZE_SEGMENT;
    let x2 = ix * SIZE_SEGMENT;
    let y2 = (iy + 1.) * SIZE_SEGMENT;

    mesh_to_brush(
        (x1 + WALL_THICKNESS, y1, 0.0).into(),
        (x2 - WALL_THICKNESS, y2, SIZE_SEGMENT).into(),
        WALL_TEXTURE.to_string(),
    )
}
