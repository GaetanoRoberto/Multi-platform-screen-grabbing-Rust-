mod image_screen;
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
use druid::{KeyEvent, Lens};
use druid::{commands, ImageBuf, Application, Color, Data, Widget, LocalizedString, WindowDesc, AppLauncher, PlatformError, widget::{Image, Label, Button, Flex}, WidgetExt, AppDelegate, DelegateCtx, WindowId, piet, LifeCycleCtx, LifeCycle, Env, RenderContext, Event, UpdateCtx, LayoutCtx, BoxConstraints, Size, PaintCtx, EventCtx, Rect, Scale, Point};
use druid::piet::ImageFormat;
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


#[derive(Clone, Data, Serialize, Deserialize, Debug, Lens)]
pub struct GrabData {
    screenshot_number: u32,
    monitor_index: usize,
    #[data(ignore)]
    image_data: Vec<u8>,
    #[data(ignore)]
    save_path: Box<Path>,
    save_format: String,
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

fn create_output_format_dropdown() -> Flex<GrabData> {
    let mut row_dropdown = Flex::row();
    row_dropdown.add_flex_child(
        Label::new("Select the output format :"),
        1.0
    );
    row_dropdown.add_default_spacer();
    row_dropdown.add_flex_child(
        DropdownSelect::new(vec![
            ("png", "png".to_string()),
            ("jpg", "jpg".to_string()),
            ("jpeg", "jpeg".to_string()),
            ("bmp", "bmp".to_string()),
            ("tiff", "tiff".to_string()),
            ("gif", "gif".to_string())
        ])
        .lens(GrabData::save_format),
        1.0
    );
    row_dropdown
}

fn build_ui() -> impl Widget<GrabData> {
    let mut ui_column = Flex::column();
    ui_column.add_flex_child(create_monitor_buttons(), 1.0);
    ui_column.add_default_spacer();
    ui_column.add_flex_child(create_output_format_dropdown(), 1.0);

    ui_column.controller(Enter)
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
        save_path: Path::new("C:\\Users\\Alessandro\\Desktop").to_path_buf().into_boxed_path(),
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

struct Enter;

impl<W: Widget<GrabData>> Controller<GrabData, W> for Enter {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &druid::Event, data: &mut GrabData, env: &Env) {
        println!("event");
        if let Event::KeyUp(key) = event {
            println!("key pressed");
            /*if key.code == Code::Enter {
                if data.new_text.trim() != "" {
                    let text = data.new_text.clone();
                    data.new_text = "".to_string();
                    data.todos.push_front(TodoItem { checked: false, text });
                }

            }*/
        }
        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &GrabData,
        env: &Env,
    ) {
        child.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, child: &mut W, ctx: &mut druid::UpdateCtx, old_data: &GrabData, data: &GrabData, env: &Env) {
        child.update(ctx, old_data, data, env)
    }
}