mod image_screen;
mod image_crop;
mod constants;

use std::{fs, thread};
use std::arch::x86_64::_addcarry_u32;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::ptr::null;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use druid::{commands, ImageBuf, Application, Color, Data, Widget, LocalizedString, WindowDesc, AppLauncher, PlatformError, widget::{Image, Label, Button, Flex}, WidgetExt, AppDelegate, DelegateCtx, WindowId, piet, LifeCycleCtx, LifeCycle, Env, RenderContext, Event, UpdateCtx, LayoutCtx, BoxConstraints, Size, PaintCtx, EventCtx, Rect, Scale, Point};
use druid::piet::ImageFormat;
use druid::platform_menus::mac::file::print;
use druid::widget::List;
use screenshots::{DisplayInfo, Screen};
use serde::{Serialize,Deserialize};
use serde_json::{to_writer,from_reader};
use image::{open, DynamicImage, ImageBuffer, Rgba, GenericImageView, load_from_memory_with_format};
use serde::de::Unexpected::Str;
use crate::image_screen::ScreenshotWidget;

#[derive(Clone, Data, Serialize, Deserialize, Debug)]
pub struct GrabData {
    screenshot_number : u32,
    monitor_index : usize,
    #[data(ignore)]
    image_data: Vec<u8>,
    #[data(ignore)]
    save_path: Box<Path>,
    save_format : String,
    press: bool,
    first_screen: bool,
    scale_factor: f64,
    #[data(ignore)]
    positions: Vec<(f64,f64)>
}

fn create_monitor_buttons() -> Flex<GrabData> {
    let screens = Screen::all().unwrap();
    let mut monitor_buttons = Flex::row();
    let mut monitor_index = 1;
    for screen in screens {
        let btn = Button::new( "📷 Monitor ".to_owned() + &monitor_index.to_string()).on_click(
            move |_ctx, _data: &mut GrabData ,_env| {
                _data.monitor_index = monitor_index - 1;
                let rect = druid::Screen::get_monitors()[_data.monitor_index].virtual_rect();
                _ctx.window().close();
                _ctx.new_window(
                    WindowDesc::new(
                        Flex::<GrabData>::row().with_child(ScreenshotWidget))
                        .show_titlebar(false)
                        .transparent(true)
                        .set_position((rect.x0,rect.y0))
                        .window_size(Size::new(screen.display_info.width as f64,screen.display_info.height as f64)));
            }
        );
        monitor_buttons = monitor_buttons.with_child(btn);
        monitor_index+=1
    }
    monitor_buttons
}

fn build_ui() -> impl Widget<GrabData> {
    Flex::column().with_child(create_monitor_buttons())
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(build_ui())
        .title("Screen grabbing Utility")
        .window_size((400.0, 300.0));

    let file = File::open("settings.json").unwrap();
    //let data: GrabData = from_reader(file).unwrap();
    let data = GrabData {
        screenshot_number: 1,
        monitor_index: 0,
        image_data: vec![],
        save_path: Path::new("C:\\Users\\Domenico\\Desktop").to_path_buf().into_boxed_path(),
        save_format: "png".to_string(),
        press: false,
        first_screen: true,
        scale_factor: 1.0,
        positions: vec![],
    };

    AppLauncher::with_window(main_window).delegate(Delegate).launch(data)
}

struct Delegate;

impl AppDelegate<GrabData> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: druid::Target,
        cmd: &druid::Command,
        _data: &mut GrabData,
        _env: &druid::Env,
    ) -> druid::Handled {
        if cmd.is(commands::CLOSE_WINDOW) {
            // TODO: set initial value for parameters who need it
            // Handle the window close event
            println!("Closing the window");
            // cancel all image data
            _data.scale_factor = 1.0;
            _data.image_data = vec![];
            let file = File::create("settings.json").unwrap();
            to_writer(file, _data).unwrap();
            // the event keep processing and the window is closed
            return druid::Handled::No;
        }
        druid::Handled::No
    }
}