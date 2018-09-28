

pub const DESIRED_FPS: u32 = 144;
pub const CHUNK_SIZE: u32 = 500;
pub const PHYSICS_EPSILON: f32 = 0.2;
pub const CAMERA_VIEW_SIZE: (u32, u32) = (1920, 1080);

// Paths

pub mod path {
    use std::path::{Path, PathBuf};

    lazy_static! {
        pub static ref RESSOURCES_DIR: PathBuf = Path::new("resources/").to_owned();

        pub static ref GAME_CONFIG_FILE: PathBuf = RESSOURCES_DIR.join("config.ron");

        pub static ref LEVELS_DIR: PathBuf = RESSOURCES_DIR.join("levels");
        pub static ref LEVEL_CONFIG_FILE: PathBuf = Path::new("level.ron").to_owned();
        pub static ref LEVEL_WORLD_DATA_FILE: PathBuf = Path::new("world.dat").to_owned();

        pub static ref MAIN_MENU_BACKGROUND_FILE: PathBuf = Path::new("/game/mainmenu.png").to_owned();
        pub static ref MAIN_MENU_LOGO_FILE: PathBuf = Path::new("/game/logo.png").to_owned();
    }
}

