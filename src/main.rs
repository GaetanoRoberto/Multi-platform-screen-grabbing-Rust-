#![windows_subsystem = "windows"]
mod image_screen;
mod constants;
mod main_gui_building;
mod handlers;
mod utilities;

use std::fs::File;
use std::io::Write;
use std::path::Path;
use druid::Lens;
use druid::{Data, WindowDesc, AppLauncher, PlatformError};
use serde::{Serialize,Deserialize};
use serde_json::from_reader;
use crate::main_gui_building::build_ui;
use crate::handlers::Delegate;
use constants::{MAIN_WINDOW_WIDTH,MAIN_WINDOW_HEIGHT};
use crate::constants::{APP_NAME, INIT_FILE};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
enum Annotation {
    None,
    Circle,
    Line,
    Cross,
    Rectangle,
    FreeLine,
    Highlighter,
    Arrow,
    Text
}

#[derive(Clone, Data, Serialize, Deserialize, Debug, Lens)]
pub struct GrabData {
    screenshot_number: u32,
    #[data(ignore)]
    image_data_old: Vec<u8>,
    #[data(ignore)]
    image_data_new: Vec<u8>,
    #[data(ignore)]
    save_path: Box<Path>,
    save_format: String,
    press: bool,
    first_screen: bool,
    scale_factors: (f64,f64),
    image_size: (f64,f64),
    #[data(ignore)]
    positions: Vec<(f64,f64)>,
    offsets: (f64,f64),
    #[data(ignore)]
    hotkey: Vec<String>,
    #[data(ignore)]
    hotkey_new: Vec<String>,
    #[data(ignore)]
    hotkey_pressed:  Vec<String>,
    set_hot_key: bool,
    delay: f64,
    input_hotkey_error: (bool,String),
    trigger_ui: bool,
    #[data(ignore)]
    annotation: Annotation,
    color: (u8,u8,u8,u8),
    text_annotation: String,
    text_size : f64,
    highlighter_width: f64,
    timer_requested: bool
}

fn main() -> Result<(), PlatformError> {
    // if settings does not exists, create it from the init hardcoded file
    let result = File::open("settings.json");
    let data: GrabData;
    match result  {
        Ok(settings) => {
            // file exists, use it
            data = from_reader(settings).unwrap();
        }
        Err(_) => {
            // file not exists, initialize data and create settings.json from init.json file
            let mut settings = File::create("settings.json").unwrap();
            data = serde_json::from_slice(INIT_FILE).unwrap();
            settings.write_all(serde_json::to_string(&data).unwrap().as_bytes()).unwrap();
        }
    }

    let main_window = WindowDesc::new(build_ui())
        .title(APP_NAME)
        .window_size((MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT)).resizable(false);

    AppLauncher::with_window(main_window).delegate(Delegate).launch(data)
}
