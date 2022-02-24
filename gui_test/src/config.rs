use directories::*;
use serde::{Deserialize, Serialize};
use std::path::*;

use window::*;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
pub enum ColorTheme {
    Dark,
    Beige,
    Light,
}

impl Default for ColorTheme {
    fn default() -> Self {
        ColorTheme::Light
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
pub enum CurvesAAMode {
    NoAntiAliasing,
    AntiAliasingX2,
    AntiAliasingX4,
}

impl Default for CurvesAAMode {
    fn default() -> Self {
        CurvesAAMode::AntiAliasingX2
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq, Copy, Clone)]
pub struct SnapOptions {
    #[serde(default)]
    pub snap_grid: bool,

    #[serde(default)]
    pub snap_endpoints: bool,

    #[serde(default)]
    pub snap_crosses: bool,

    #[serde(default)]
    pub snap_centers: bool,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
pub struct ConfigFontSize(pub i32);

impl ConfigFontSize {
    pub fn adjusted(self) -> Self {
        Self(std::cmp::max(10, std::cmp::min(self.0, 100)))
    }
}

impl Default for ConfigFontSize {
    fn default() -> Self {
        Self((get_screen_resolution().0 as f32).sqrt() as i32 / 2).adjusted()
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub font_size: ConfigFontSize,

    #[serde(default)]
    pub color_theme: ColorTheme,

    #[serde(default)]
    pub curves_aa_mode: CurvesAAMode,

    #[serde(default)]
    pub font_aa_mode: application::font::FontAntiAliasingMode,

    #[serde(default)]
    pub show_grid: bool,

    #[serde(default)]
    pub snap_options: SnapOptions,

    #[serde(default)]
    pub window_position: Option<WindowPosition>,
}

pub fn get_project_dir() -> Option<ProjectDirs> {
    ProjectDirs::from("ru", "T4r4sB", "OtCAD")
}

pub fn load_config() -> Option<Config> {
    let config_dir = get_project_dir()?.config_dir().to_path_buf();
    let config_file = Path::join(&config_dir, "config.json");
    std::fs::create_dir_all(&config_dir).ok()?;

    let file = std::fs::File::open(config_file).ok()?;
    let reader = std::io::BufReader::new(file);
    let result = serde_json::from_reader(reader).ok()?;
    result
}

pub fn save_config(config: &Config) -> Option<()> {
    let config_dir = get_project_dir()?.config_dir().to_path_buf();
    let config_file = Path::join(&config_dir, "config.json");
    std::fs::create_dir_all(&config_dir).ok()?;
    serde_json::to_writer_pretty(&std::fs::File::create(config_file).ok()?, &config).ok()?;
    Some(())
}
