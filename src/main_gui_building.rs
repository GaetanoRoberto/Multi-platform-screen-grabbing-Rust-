use std::{fmt, fs};
use std::error::Error;
use std::fmt::Debug;
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use druid::widget::{Button, Controller, Flex, Label, TextBox, ValueTextBox};
use druid::{Color, Command, Env, Event, EventCtx, KbKey, Selector, Size, Widget, WidgetExt, WindowDesc};
use druid::text::{Validation, ValidationError};
use druid_widget_nursery::{DropdownSelect, WidgetExt as OtherWidgetExt};
use druid::text::{Formatter, Selection};
use druid_widget_nursery::stack_tooltip::tooltip_state_derived_lenses::data;
use screenshots::Screen;
use serde_json::from_reader;
use crate::constants::{BUTTON_HEIGHT, BUTTON_WIDTH,MAIN_WINDOW_WIDTH,MAIN_WINDOW_HEIGHT};
use crate::GrabData;
use crate::image_screen::ScreenshotWidget;
use crate::handlers::{Enter, NumericTextBoxController};
use crate::input_field::PositiveNumberFormatter;

pub fn start_screening(ctx: &mut EventCtx, monitor_index: usize) {
    let screen = Screen::all().unwrap()[monitor_index];
    let rect = druid::Screen::get_monitors()[monitor_index].virtual_rect();
    ctx.window().close();
    ctx.new_window(
        WindowDesc::new(
            Flex::<GrabData>::row().with_child(ScreenshotWidget))
            .show_titlebar(false)
            .transparent(true)
            .set_position((rect.x0,rect.y0))
            .window_size(Size::new(screen.display_info.width as f64,screen.display_info.height as f64)));
}

fn create_monitor_buttons() -> Flex<GrabData> {
    let screens = Screen::all().unwrap();
    let mut monitor_buttons = Flex::row();
    let mut monitor_index = 1;
    for screen in screens {
        let btn = Button::new( "📷 Monitor ".to_owned() + &monitor_index.to_string()).on_click(
            move |_ctx, _data: &mut GrabData ,_env| {
                _data.monitor_index = monitor_index - 1;
                start_screening(_ctx,_data.monitor_index);
            }
        );
        monitor_buttons = monitor_buttons.with_child(btn);
        monitor_index+=1
    }
    monitor_buttons
}

fn create_output_format_dropdown() -> Flex<GrabData> {
    let file = File::open("settings.json").unwrap();
    let data: GrabData = from_reader(file).unwrap();

    let standard_formats = vec![
        ("png".to_string(), "png".to_string()),
        ("jpg".to_string(), "jpg".to_string()),
        ("jpeg".to_string(), "jpeg".to_string()),
        ("bmp".to_string(), "bmp".to_string()),
        ("tiff".to_string(), "tiff".to_string()),
        ("gif".to_string(), "gif".to_string())
    ];

    let mut build_formats = vec![(data.save_format.clone(),data.save_format.clone())];

    for format in standard_formats {
        if format.0 != data.save_format {
            build_formats.push(format);
        }
    }
    let mut row_dropdown = Flex::row();
    row_dropdown.add_flex_child(
        Label::new("Select the output format :"),
        1.0
    );
    row_dropdown.add_default_spacer();
    row_dropdown.add_flex_child(
        DropdownSelect::new(build_formats)
            .lens(GrabData::save_format),
        1.0
    );
    row_dropdown
}

fn create_hotkey_ui() -> impl Widget<GrabData> {
    let mut fusion = Flex::column();
    let mut ui_column = Flex::row();
    let mut ui2_column = Flex::row();

    ui2_column.add_flex_child(Label::dynamic(|data: &GrabData, _| "Selected Hotkeys/Timer Monitor: ".to_owned() + &(data.monitor_index.clone() + 1).to_string()), 1.0);

    ui_column.add_default_spacer();
    ui_column.add_flex_child(Button::dynamic(|data: & GrabData, _env| {
        if data.set_hot_key {
            "Set Hotkeys".to_string()
        } else {
            "Edit Hotkeys".to_string()
        }
    }).on_click(
        move |_ctx, _data: &mut GrabData, _env| {
            _data.set_hot_key = !_data.set_hot_key;
            if _data.set_hot_key == true {
                // from false to true, i want to edit hotkeys, i start from scratch with empty vector combination
                _data.hotkey.clear();
            }
        }
    ), 1.0);

    ui_column.add_default_spacer();
    ui_column.add_flex_child(Label::new(|data: &GrabData, _env: &_| {
        data.hotkey.join(" + ")
    }), 1.0);
    let screens = Screen::all().unwrap();
    let mut monitor_buttons = Flex::row();
    let mut monitor_index = 1;
    for screen in screens {
        let btn = Button::new( "Monitor ".to_owned() + &monitor_index.to_string()).on_click(
            move |_ctx, _data: &mut GrabData ,_env| {
                _data.monitor_index = monitor_index - 1;
            });
        monitor_buttons = monitor_buttons.with_child(btn);
        monitor_index+=1
    }
    ui_column.add_default_spacer();
    ui_column.add_flex_child(monitor_buttons, 1.0);

    fusion.add_flex_child(ui2_column,1.0);
    fusion.add_default_spacer();
    fusion.add_flex_child(ui_column,1.0);

    fusion
}

pub fn create_save_cancel_buttons() -> impl Widget<GrabData> {
    let save_button = Button::new("Save").on_click(move |_ctx, _data: &mut GrabData ,_env| {
        if !_data.image_data.is_empty() {
            fs::write(format!("{}\\Screen{}.{}", _data.save_path.to_str().unwrap(), _data.screenshot_number, _data.save_format), _data.image_data.clone()).unwrap();
        }
        if _data.screenshot_number == u32::MAX {
            _data.screenshot_number = 0;
        } else {
            _data.screenshot_number+=1;
        }
        // cancel all image data
        _data.image_data = vec![];
        _data.first_screen = true;
        _ctx.window().close();
        _ctx.new_window(WindowDesc::new(build_ui())
            .title("Screen grabbing Utility")
            .window_size((MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT)));
    }).fix_size(BUTTON_WIDTH, BUTTON_HEIGHT);

    let cancel_button = Button::new("Cancel").on_click(move |_ctx, _data: &mut GrabData ,_env| {
        // cancel all image data
        _data.image_data = vec![];
        _data.first_screen = true;
        _ctx.window().close();
        _ctx.new_window(WindowDesc::new(build_ui())
            .title("Screen grabbing Utility")
            .window_size((MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT)));
    }).fix_size(BUTTON_WIDTH, BUTTON_HEIGHT);

    Flex::row().with_child(save_button).with_child(cancel_button)
}

fn build_path_dialog() -> impl Widget<GrabData> {
    let path_label = Label::dynamic(|data: &GrabData, _env: &_| "Current Path: ".to_owned() + data.save_path.to_str().unwrap() );

    let change_path_button = Button::new("📁").on_click(|ctx, data: &mut GrabData, _env| {
        let result = nfd::open_pick_folder(Some(data.save_path.to_str().unwrap())).ok().unwrap();
        match result {
            nfd::Response::Okay(path) => {
                data.trigger_ui = !data.trigger_ui;
                data.save_path = Path::new(path.as_str()).to_path_buf().into_boxed_path();
            },
            _ => (),
        };
        println!("{:?}",data.save_path);
    });

    let mut ui_row = Flex::row();
    ui_row.add_flex_child(path_label, 2.0);
    ui_row.add_default_spacer();
    ui_row.add_flex_child(change_path_button, 1.0);

    ui_row
}

fn create_timer_ui() -> impl Widget<GrabData> {
    let label = Label::new("Insert here the Delay in Seconds:");

    let textbox = ValueTextBox::new(TextBox::new(), PositiveNumberFormatter)
        .update_data_while_editing(true)
        .lens(GrabData::delay);

    let error_label = Label::dynamic(|data: &GrabData, _: &Env| {
        if data.input_error.0 {
            data.input_error.1.clone()
        } else {
            String::new()
        }
    })
    .with_text_color(Color::rgb(0.8, 0.0, 0.0));

    let start_timer_btn = Button::new("Start Timer").on_click(|ctx, data: &mut GrabData, _env| {
        //start the timer
        let delay_value = data.delay.parse::<u64>();
        if delay_value.is_ok() {
            println!("{:?}",Duration::from_secs(delay_value.clone().unwrap()));
            data.timer_id = ctx.request_timer(Duration::from_secs(delay_value.unwrap())).into_raw();
        } else {
            data.input_error.0 = true;
            data.input_error.1 = "Empty Input: Insert a Positive Number in the Field.".to_string()
        }
    });

    let mut ui_row = Flex::row();
    ui_row.add_flex_child(label, 2.0);
    ui_row.add_default_spacer();
    ui_row.add_flex_child(textbox, 1.0);
    ui_row.add_default_spacer();
    ui_row.add_flex_child(start_timer_btn, 1.0);

    Flex::column().with_child(ui_row).with_child(error_label).controller(NumericTextBoxController)
}

pub(crate) fn build_ui() -> impl Widget<GrabData> {
    let mut ui_column = Flex::column();
    ui_column.add_flex_child(create_monitor_buttons(),1.0);
    ui_column.add_default_spacer();
    ui_column.add_flex_child(create_output_format_dropdown(),1.0);
    ui_column.add_default_spacer();
    ui_column.add_flex_child(create_hotkey_ui(),1.0);
    ui_column.add_default_spacer();
    ui_column.add_flex_child(build_path_dialog(),1.0);
    ui_column.add_default_spacer();
    ui_column.add_flex_child(create_timer_ui(),1.0);

    ui_column.controller(Enter)
    // Flex::column()
}