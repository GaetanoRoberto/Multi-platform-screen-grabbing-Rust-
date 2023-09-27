use std::{fmt, fs, thread};
use std::borrow::Cow;
use std::fmt::Debug;
use std::fs::File;
use std::time::Duration;
use druid::widget::{Button, Controller, Flex, Image, Label, TextBox, ValueTextBox, ZStack};
use druid::{Color, Command, Env, Event, EventCtx, ImageBuf, KbKey, Selector, Size, Widget, WidgetExt, WindowConfig, WindowDesc};
use druid::piet::ImageFormat;
use druid_widget_nursery::{AdvancedSlider, DropdownSelect, WidgetExt as OtherWidgetExt};
use image::{DynamicImage, EncodableLayout, load_from_memory_with_format, Rgba};
use imageproc::drawing::draw_text;
use screenshots::Screen;
use serde_json::{from_reader, to_writer};
use crate::constants::{BUTTON_HEIGHT, BUTTON_WIDTH, MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT, OPACITY};
use crate::{Annotation, GrabData};
use crate::image_screen::{image_to_buffer, load_image, ScreenshotWidget};
use crate::handlers::{Enter, NumericTextBoxController};
use crate::input_field::PositiveNumberFormatter;
use native_dialog::{FileDialog};
use rusttype::Font;
use tokio;

pub fn start_screening(ctx: &mut EventCtx, monitor_index: usize) {
    let screen = Screen::all().unwrap()[monitor_index];
    let rect = druid::Screen::get_monitors()[monitor_index].virtual_rect();
    ctx.window().close();
    ctx.new_window(
        WindowDesc::new(
            Flex::<GrabData>::row().with_child(ScreenshotWidget).background(Color::rgba(0.0,0.0,0.0, OPACITY)))
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
    let file = File::open("settings.json").unwrap();
    let data: GrabData = from_reader(file).unwrap();

    let mut fusion = Flex::column();
    let mut ui_column = Flex::row();
    let mut ui2_column = Flex::row();

    ui2_column.add_flex_child(Label::new("Selected Hotkeys/Timer Monitor: "), 1.0);
    ui2_column.add_default_spacer();

    let mut monitors = vec![];
    let screens = Screen::all().unwrap();
    for monitor_index in 0..screens.len() {
        monitors.push(((monitor_index+1).to_string(),monitor_index))
    }

    let mut build_monitors = vec![((data.monitor_index+1).to_string(),data.monitor_index)];

    for monitor in monitors {
        if monitor.1 != data.monitor_index {
            build_monitors.push(monitor);
        }
    }

    ui2_column.add_flex_child(
        DropdownSelect::new(build_monitors)
            .lens(GrabData::monitor_index),
        1.0
    );

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
    /*let screens = Screen::all().unwrap();
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
    ui_column.add_flex_child(monitor_buttons, 1.0);*/

    fusion.add_flex_child(ui2_column,1.0);
    fusion.add_default_spacer();
    fusion.add_flex_child(ui_column,1.0);

    fusion
}

pub fn create_save_cancel_clipboard_buttons() -> impl Widget<GrabData> {
    let save_button = Button::new("Save").on_click(move |_ctx, _data: &mut GrabData ,_env| {
        if !_data.image_data_old.is_empty() {
            // save file
            let result = FileDialog::new()
                .set_filename(format!("Screen{}.{}",_data.screenshot_number, _data.save_format).as_str())
                .add_filter("", &[_data.save_format.as_str()])
                .set_location(_data.save_path.to_str().unwrap())
                .show_save_single_file()
                .unwrap();
            match result {
                Some(path) => {
                    // The user selected a file to save.
                    // save the file and increment the screenshot counter
                    // if the user selected a custom filename, no need to increment the automatic inner counter
                    if path.file_name().unwrap().to_string_lossy().to_string().contains("Screen") {
                        println!("Increment no filename changed");
                        if _data.screenshot_number == u32::MAX {
                            _data.screenshot_number = 0;
                        } else {
                            _data.screenshot_number+=1;
                        }
                    }
                    fs::write(path, _data.image_data_old.clone()).unwrap();
                    // cancel all image data
                    _data.image_data_old = vec![];
                    _data.first_screen = true;
                }
                None => {
                    // The user canceled the dialog.
                    println!("Dialog Cancelled");
                }
            }
        }
        // if handles the else, a message window saying no file, but the save button appears only when there is an image for now
        _ctx.window().close();
        _ctx.new_window(WindowDesc::new(build_ui())
            .title("Screen grabbing Utility")
            .window_size((MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT)));
    }).fix_size(BUTTON_WIDTH, BUTTON_HEIGHT);

    let cancel_button = Button::new("Cancel").on_click(move |_ctx, _data: &mut GrabData ,_env| {
        // cancel all image data
        _data.image_data_old = vec![];
        _data.first_screen = true;
        _ctx.window().close();
        _ctx.new_window(WindowDesc::new(build_ui())
            .title("Screen grabbing Utility")
            .window_size((MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT)));
    }).fix_size(BUTTON_WIDTH, BUTTON_HEIGHT);

    let clipboard_button = Button::new("Copy to Clipboard").on_click(move |_ctx, _data: &mut GrabData ,_env| {
        // copy to the clipboard
        let image = load_from_memory_with_format(&_data.image_data_old, image::ImageFormat::Png).unwrap().to_rgba8();
        let  mut clipboard = arboard::Clipboard::new().unwrap();

        let img = arboard::ImageData {
            width: image.width() as usize,
            height: image.height() as usize,
            bytes: Cow::from(image.as_bytes())
        };
        clipboard.set_image(img).unwrap();

    }).fix_size(BUTTON_WIDTH * 2.0, BUTTON_HEIGHT);

    let mut ui_row = Flex::row();

    ui_row.add_flex_child(save_button, 0.1);
    ui_row.add_default_spacer();
    ui_row.add_flex_child(cancel_button, 0.1);
    ui_row.add_default_spacer();
    ui_row.add_flex_child(Flex::row().with_child(clipboard_button), 0.1);

    ui_row
}

fn build_path_dialog() -> impl Widget<GrabData> {
    let path_label = Label::dynamic(|data: &GrabData, _env: &_| "Current Path: ".to_owned() + data.save_path.to_str().unwrap() );

    let change_path_button = Button::new("📁").on_click(|ctx, data: &mut GrabData, _env| {
        let result = FileDialog::new()
            .set_location(data.save_path.to_str().unwrap())
            .show_open_single_dir();
        match result {
            Ok(opt_path) => {
                match opt_path {
                    Some(path) => {
                        // set the new folder path and trigger ui refresh
                        data.trigger_ui = !data.trigger_ui;
                        data.save_path = path.into_boxed_path();
                    }
                    None => {
                        println!("Dialog Cancelled");
                    }
                }
            }
            Err(_) => {
                panic!("Error in Setting the Path");
            }
        }
    });

    let mut ui_row = Flex::row();
    ui_row.add_flex_child(path_label, 2.0);
    ui_row.add_default_spacer();
    ui_row.add_flex_child(change_path_button, 1.0);

    ui_row
}

#[tokio::main]
pub async fn timer_handling(ctx: &mut EventCtx,monitor_index: usize, time: u64) {
    // Sleep for time seconds
    tokio::time::sleep(Duration::from_secs(time)).await;
    // take the screenshot
    start_screening(ctx,monitor_index);
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
        timer_handling(ctx,data.monitor_index,data.delay.parse::<u64>().unwrap());
    });

    let mut ui_row = Flex::row();
    ui_row.add_flex_child(label, 2.0);
    ui_row.add_default_spacer();
    ui_row.add_flex_child(textbox, 1.0);
    ui_row.add_default_spacer();
    ui_row.add_flex_child(start_timer_btn, 1.0);

    Flex::column().with_child(ui_row).with_child(error_label).controller(NumericTextBoxController)
}

pub fn create_annotation_buttons() -> impl Widget<GrabData> {
    let mut ui_row1 = Flex::row();
    let mut ui_row2 = Flex::row();
    let file = File::open("settings.json").unwrap();
    let data: GrabData = from_reader(file).unwrap();
    // beizer curve ellisse

    ui_row1.add_flex_child(Button::new("✂").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::None;
        create_edit_window(ctx,data);
    }), 1.0);
    ui_row1.add_default_spacer();
    ui_row1.add_flex_child(Button::new("◯").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::Circle;
        create_edit_window(ctx,data);
    }),1.0);
    ui_row1.add_default_spacer();
    ui_row1.add_flex_child(Button::new("╱").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::Line;
        create_edit_window(ctx,data);
    }), 1.0);
    ui_row1.add_default_spacer();
    ui_row1.add_flex_child(Button::new("✖").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::Cross;
        create_edit_window(ctx,data);
    }), 1.0);
    ui_row1.add_default_spacer();
    ui_row1.add_flex_child(Button::new("▢").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::Rectangle;
        create_edit_window(ctx,data);
    }), 1.0);
    ui_row1.add_default_spacer();


    ui_row2.add_flex_child(Button::new("〜").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::FreeLine;
        create_edit_window(ctx,data);
    }), 1.0);
    ui_row2.add_default_spacer();
    ui_row2.add_flex_child(Button::new("⇗").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::Arrow;
        create_edit_window(ctx,data);
    }), 1.0);
    ui_row2.add_default_spacer();
    ui_row2.add_flex_child(Button::new("A").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::Text;
        create_edit_window(ctx,data);
    }), 1.0);
    ui_row2.add_default_spacer();
    ui_row2.add_flex_child(Button::new("💄").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::Highlighter;
        create_edit_window(ctx,data);
    }), 1.0);
    ui_row2.add_default_spacer();
    ui_row2.add_flex_child(Button::from_label(Label::new("⬤").with_text_color(Color::rgba8(data.color.0,data.color.1,data.color.2,data.color.3))).on_click(|ctx, data: &mut GrabData, _env| {
        //data.annotation = Annotation::Circle;
        ctx.new_sub_window(WindowConfig::default().window_size((100.0,100.0)).show_titlebar(false), create_color_buttons(), data.clone(), _env.clone());
    }), 1.0);

    Flex::column().with_child(ui_row1).with_child(ui_row2)
}

pub fn create_color_buttons() -> impl Widget<GrabData> {
    // 12 colors 4 x 3
    let mut ui_col = Flex::column();
    // giallo verde blu viola rosso arancione rosa nero bianco marrone grigio
    let orange = Color::rgba8(255, 165, 0, 255);
    let pink = Color::rgba8(255, 192, 203, 255);
    let brown= Color::rgba8(139, 69, 19, 255);
    let colors: [Color; 12] = [Color::RED, Color::GREEN, Color::BLUE, Color::YELLOW,orange, pink,brown, Color::BLACK,Color::WHITE, Color::GRAY, Color::PURPLE, Color::FUCHSIA];

    for chunk in colors.chunks(4) {
        let mut ui_row = Flex::row();
        for &color in chunk {
            ui_row.add_flex_child(
                Button::from_label(Label::new("⬤").with_text_color(color))
                    .on_click(move |ctx, data: &mut GrabData, _env| {
                        // Change the color and save it
                        data.color = color.as_rgba8();

                        let file = File::create("settings.json").unwrap();
                        to_writer(file, data).unwrap();
                        ctx.window().close();
                        //update_color_callback(color);

                    }),1.0,);
        }
        // Aggiungo la riga al layout
        ui_col.add_flex_child(ui_row, 1.0);
        ui_col.add_default_spacer();
    }
    ui_col
}

pub fn create_edit_window_widgets(data: &GrabData) -> impl Widget<GrabData> {
    let mut ui_column = Flex::column();
    let mut ui_row1 = Flex::row();

    let approve = Button::new("✔").on_click(|ctx, data: &mut GrabData ,_env| {
        // modified, so new image become the old image, ready to be saved
        if !data.image_data_new.is_empty() {
            data.image_data_old = data.image_data_new.clone();
        }
        // clear the new image
        data.image_data_new = vec![];
        // reset annotation
        data.annotation = Annotation::None;
        // return in the selection window
        create_selection_window(ctx,data);
    });
    let reject = Button::new("✖").on_click(|ctx, data: &mut GrabData ,_env| {
        // discard the new image
        data.image_data_new = vec![];
        // reset annotation
        data.annotation = Annotation::None;
        // return in the selection window
        create_selection_window(ctx,data);
    });

    ui_row1.add_flex_child(approve,1.0);
    ui_row1.add_default_spacer();
    ui_row1.add_flex_child(reject,1.0);
    ui_row1.add_default_spacer();

    ui_column.add_flex_child(ui_row1, 1.0);
    ui_column.add_default_spacer();

    if data.annotation == Annotation::Text {
        // add also text handling widgets
        let add_text = Button::new("Add Text").on_click(|ctx, data: &mut GrabData, _env| {
            // draw text
            let image = load_image(data);

            let font_data: &[u8] = include_bytes!("../OpenSans-Semibold.ttf");
            //fs::read(Path::new(format!("{}{}",std::env::current_dir().unwrap().to_str().unwrap(),"\\OpenSans-Semibold.ttf").as_str())).unwrap().as_slice();
            let font: Font<'static> = Font::try_from_bytes(font_data).unwrap();
            // draw line with first and last position, then clear the vector
            let text_image = DynamicImage::from(
                draw_text(&image,
                          Rgba([data.color.0, data.color.1, data.color.2, data.color.3]),
                          data.positions[0].0 as i32, data.positions[0].1 as i32,
                          rusttype::Scale::uniform(data.text_size as f32), &font, data.text_annotation.as_str()));
            // save the modified version of the image
            data.image_data_new = image_to_buffer(text_image);

            // empty position vector, not done in ScreenshotWidget
            data.positions = vec![];

            // recreate the window
            create_edit_window(ctx,data);
        });
        let text_input = TextBox::new().lens(GrabData::text_annotation);
        let text_font_size = druid::widget::Stepper::new()
            .with_range(10.0, 50.0)
            .with_step(1.0)
            .lens(GrabData::text_size);
        let font_size = Label::dynamic(|data: &GrabData, _env: &_| "Font Size: ".to_owned() +  data.text_size.to_string().as_str() );

        ui_column.add_flex_child(add_text,1.0);
        ui_column.add_default_spacer();
        ui_column.add_flex_child(text_input,1.0);
        ui_column.add_default_spacer();
        ui_column.add_flex_child(Flex::row().with_child(text_font_size).with_child(font_size), 1.0);
        ui_column.add_default_spacer();
    }

    ui_column
}

pub fn create_edit_window(ctx: &mut EventCtx, data: &mut GrabData) {

    let image = load_image(data).to_rgba8();

    let rect = druid::Screen::get_monitors()[data.monitor_index].virtual_rect();
    let image_buf = ImageBuf::from_raw(
        image.clone().into_raw(),
        ImageFormat::RgbaSeparate,
        image.clone().width() as usize,
        image.clone().height() as usize,
    );

    ctx.window().close();
    ctx.new_window(
        WindowDesc::new(
            Flex::column().with_child(
                Flex::column()
                    .with_child(
                        ZStack::new(Image::new(image_buf))
                            .with_centered_child(ScreenshotWidget)
                    )
            ).with_child(create_edit_window_widgets(data)))
            .set_position((rect.x0,rect.y0))
            .window_size(Size::new( image.width() as f64 * data.scale_factor,(image.height() as f64 * data.scale_factor + BUTTON_HEIGHT * 6.0)))
            .resizable(true))
}


pub fn create_selection_window(ctx: &mut EventCtx, data: &mut GrabData) {

    let image = load_image(data).to_rgba8();

    let rect = druid::Screen::get_monitors()[data.monitor_index].virtual_rect();
    let image_buf = ImageBuf::from_raw(
        image.clone().into_raw(),
        ImageFormat::RgbaSeparate,
        image.clone().width() as usize,
        image.clone().height() as usize,
    );

    ctx.window().close();
    ctx.new_window(
        WindowDesc::new(
            Flex::column().with_child(
                Flex::column()
                    .with_child(Image::new(image_buf))
            ).with_child(Flex::column().with_child(create_save_cancel_clipboard_buttons())
                .with_child(create_annotation_buttons())))
            .set_position((rect.x0,rect.y0))
            .window_size(Size::new( image.width() as f64 * data.scale_factor,(image.height() as f64 * data.scale_factor + BUTTON_HEIGHT * 6.0)))
            .resizable(true))
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