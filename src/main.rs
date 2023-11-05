#![windows_subsystem = "windows"]
mod image_screen;
mod constants;
mod main_gui_building;
mod handlers;
mod input_field;
mod utilities;

use std::env;
use std::arch::x86_64::_addcarry_u32;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::ptr::null;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use druid::{HotKey, KeyEvent, Lens, TimerToken};
use druid::{commands, ImageBuf, Application, Data, Widget, LocalizedString, WindowDesc, AppLauncher, PlatformError, widget::{Image, Label, Button, Flex}, WidgetExt, AppDelegate, DelegateCtx, WindowId, piet, LifeCycleCtx, LifeCycle, Env, RenderContext, Event, UpdateCtx, LayoutCtx, BoxConstraints, Size, PaintCtx, EventCtx, Rect, Scale, Point};
use druid::keyboard_types::Key;
use druid::kurbo::common::factor_quartic_inner;
use druid::piet::{ImageFormat, TextStorage};
use druid::platform_menus::mac::file::print;
use druid::platform_menus::win::file::print_preview;
use druid::widget::{Controller, List, RadioGroup, TextBox};
use druid_widget_nursery::{DropdownSelect};
use grab_data_derived_lenses::save_format;
use screenshots::{DisplayInfo, Screen};
use serde::{Serialize,Deserialize};
use serde_json::{to_writer,from_reader};
use image::{open, DynamicImage, ImageBuffer, Rgba, GenericImageView, load_from_memory_with_format};
use serde::de::Unexpected::Str;
use druid_widget_nursery::Dropdown;
use druid_widget_nursery::dropdown::{DROPDOWN_CLOSED, DROPDOWN_SHOW};
use druid_widget_nursery::stack_tooltip::tooltip_state_derived_lenses::data;
use crate::image_screen::ScreenshotWidget;
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
    monitor_index: usize,
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
    highlighter_width: f64
}

fn main() -> Result<(), PlatformError> {
    // if settings does not exists, create it from the init hardcoded file
    let result = File::open("settings.json");
    let mut data: GrabData;
    match result  {
        Ok(settings) => {
            // file exists, use it
            println!("exists");
            data = from_reader(settings).unwrap();
        }
        Err(_) => {
            // file not exists, initialize data and create settings.json from init.json file
            println!("not exists");
            let mut settings = File::create("settings.json").unwrap();
            data = serde_json::from_slice(INIT_FILE).unwrap();
            settings.write_all(serde_json::to_string(&data).unwrap().as_bytes()).unwrap();
        }
    }

    let main_window = WindowDesc::new(build_ui())
        .title(APP_NAME)
        .window_size((MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT)).resizable(false);

    let file = File::open("settings.json").unwrap();
    let data: GrabData = from_reader(file).unwrap();

    /*let data = GrabData {
        screenshot_number: 1,
        monitor_index: 0,
        image_data_old: vec![],
        image_data_new: vec![],
        save_path: env::current_dir().expect("Failed to get current directory").into_boxed_path(),
        save_format: "png".to_string(),
        press: false,
        first_screen: true,
        scale_factor: 1.0,
        positions: vec![],
        hotkey: vec!["Alt".to_string(),"5".to_string()],
        hotkey_sequence: 0,
        set_hot_key: false,
        delay: "".to_string(),
        delay_length: 0,
        input_timer_error: (false,"Invalid Input: Only Positive Number are Allowed.".to_string()),
        trigger_ui: false,
        annotation: Annotation::None,
        color: (255,255,255,255),
        text_annotation: "".to_string(),
        text_size : 0.0
    };*/
    AppLauncher::with_window(main_window).delegate(Delegate).launch(data)
}
