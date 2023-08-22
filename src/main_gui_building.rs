use std::fs;
use std::fs::File;
use druid::widget::{Button, Flex, Label};
use druid::{Size, Widget, WidgetExt, WindowDesc};
use druid_widget_nursery::DropdownSelect;
use screenshots::Screen;
use serde_json::from_reader;
use crate::constants::{BUTTON_HEIGHT, BUTTON_WIDTH};
use crate::GrabData;
use crate::image_screen::ScreenshotWidget;
use crate::handlers::Enter;

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

    ui2_column.add_flex_child(Label::dynamic(|data: &GrabData, _| "Selected Hotkeys Monitor: ".to_owned() + &(data.monitor_index.clone() + 1).to_string()), 1.0);

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
            fs::write(format!("Screen{}.{}",_data.screenshot_number,_data.save_format), _data.image_data.clone()).unwrap();
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
            .window_size((400.0, 300.0)));
    }).fix_size(BUTTON_WIDTH, BUTTON_HEIGHT);

    let cancel_button = Button::new("Cancel").on_click(move |_ctx, _data: &mut GrabData ,_env| {
        // cancel all image data
        _data.image_data = vec![];
        _data.first_screen = true;
        _ctx.window().close();
        _ctx.new_window(WindowDesc::new(build_ui())
            .title("Screen grabbing Utility")
            .window_size((400.0, 300.0)));
    }).fix_size(BUTTON_WIDTH, BUTTON_HEIGHT);

    Flex::row().with_child(save_button).with_child(cancel_button)
}

pub(crate) fn build_ui() -> impl Widget<GrabData> {
    let mut ui_column = Flex::column();
    ui_column.add_flex_child(create_monitor_buttons(),1.0);
    ui_column.add_default_spacer();
    ui_column.add_flex_child(create_output_format_dropdown(),1.0);
    ui_column.add_default_spacer();
    ui_column.add_flex_child(create_hotkey_ui(),1.0);

    ui_column.controller(Enter)
    // Flex::column()
}