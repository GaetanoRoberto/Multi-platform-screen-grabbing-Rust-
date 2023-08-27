use std::borrow::Cow;
use std::cmp::{max, min};
use std::fs;
use std::fs::{create_dir_all, File};
use std::path::Path;
use druid::{Application, BoxConstraints, Clipboard, ClipboardFormat, Color as druidColor, Color, commands, Cursor, Env, Event, EventCtx, FormatId, ImageBuf, LayoutCtx, LifeCycle, LifeCycleCtx, MouseButton, Point, Rect, Scale, Screen, Size, UnitPoint, UpdateCtx, Vec2, Widget, WidgetExt, WindowConfig, WindowDesc};
use druid::piet::{ImageFormat, InterpolationMode};
use druid::piet::PaintBrush::Fixed;
use druid::platform_menus::mac::file::print;
use druid::widget::{ZStack, Button, Container, Flex, Image, SizedBox, FillStrat, Label, ClipBox, Controller};
use image::{DynamicImage, EncodableLayout, ImageBuffer, load_from_memory_with_format, Rgba};
use image::imageops::FilterType;
use imageproc::drawing::{Canvas, draw_cross, draw_filled_rect, draw_hollow_circle, draw_hollow_circle_mut, draw_hollow_rect, draw_line_segment, draw_polygon, draw_text};
use serde_json::{from_reader, to_writer};
use crate::{main_gui_building::build_ui, constants, GrabData, Annotation};
use constants::{BUTTON_HEIGHT,BUTTON_WIDTH,LIMIT_PROPORTION,SCALE_FACTOR};
use crate::main_gui_building::create_save_cancel_buttons;
use rusttype::Font;

pub struct ScreenshotWidget;

impl Widget<GrabData> for ScreenshotWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut GrabData, _env: &Env) {
        let mut min_x = 0;
        let mut min_y = 0;
        let mut max_x = 0;
        let mut max_y = 0;

        if let Event::MouseDown(mouse_event) = event {
            if mouse_event.button.is_left() {
                data.press = true;
            }
            //ctx.set_cursor(&Cursor::Crosshair);
        }

        if let Event::MouseMove(mouse_event) = event {
            ctx.set_cursor(&Cursor::Crosshair);
            //println!("{:?}",(mouse_event.pos.x,mouse_event.pos.y));
            if data.press && data.first_screen {
                data.positions.push((mouse_event.window_pos.x,mouse_event.window_pos.y));
            }
            if data.press && !data.first_screen {
                let mut image = load_from_memory_with_format(&data.image_data, image::ImageFormat::Png)
                    .expect("Failed to load image from memory");
                // Calculate the offset to center mouse positions in the Image
                let widget_size = ctx.size();
                let image_width = image.width() as f64 * data.scale_factor;
                let image_height = image.height() as f64 * data.scale_factor;
                let x_offset = (widget_size.width - image_width) / 2.0;
                let y_offset = (widget_size.height - image_height) / 2.0;
                // Adjust mouse coordinates
                let mut centered_pos = mouse_event.pos - Vec2::new(x_offset, y_offset);
                centered_pos.x = centered_pos.x / data.scale_factor;
                centered_pos.y = centered_pos.y / data.scale_factor;
                data.positions.push(<(f64, f64)>::from(centered_pos));
            }
        }

        if let Event::MouseUp(_) = event {
            data.press = false;
            //println!("{:?}",data.positions);
            if !data.positions.is_empty() {
                let screen = screenshots::Screen::all().unwrap()[data.monitor_index];

                /*let (min_x2, max_y2) = data.positions.iter().cloned().fold(
                    (f64::INFINITY, f64::NEG_INFINITY),
                    |(min_x, max_y), (x, y)| (min_x.min(x), max_y.max(y)),
                );

                let (max_x2, min_y2) = data.positions.iter().cloned().fold(
                    (f64::NEG_INFINITY, f64::INFINITY),
                    |(max_x, min_y), (x, y)| (max_x.max(x), min_y.min(y)),
                );*/

                let (mut min_x2,mut max_y2) = (0.0,0.0);
                let (mut max_x2,mut min_y2) = (0.0,0.0);
                let (p1x,p1y) = data.positions[0];
                let (p2x,p2y) = data.positions[data.positions.len() - 1];

                if p1x < p2x && p1y < p2y {
                    // p1 smaller than p2
                    min_x2 = p1x;
                    min_y2 = p1y;
                    max_x2 = p2x;
                    max_y2 = p2y;
                } else if p1x > p2x && p1y > p2y {
                    // p2 smaller than p1
                    min_x2 = p2x;
                    min_y2 = p2y;
                    max_x2 = p1x;
                    max_y2 = p1y;
                } else if p1x < p2x && p1y > p2y {
                    // partenza in basso a sx
                    min_x2 = p1x;
                    min_y2 = p2y;
                    max_x2 = p2x;
                    max_y2 = p1y;
                } else if p1x > p2x && p1y < p2y {
                    // partenza in alto a dx
                    min_x2 = p2x;
                    min_y2 = p1y;
                    max_x2 = p1x;
                    max_y2 = p2y;
                }

                let scale_factor_x = ctx.scale().x();
                let scale_factor_y = ctx.scale().y();
                min_x = (min_x2 as f64 * scale_factor_x) as i32;
                max_x = (max_x2 as f64 * scale_factor_x) as i32;

                if !data.first_screen {
                    //min_y = ((min_y2 as f64 * scale_factor_y)+35.0) as i32;
                    //max_y = ((max_y2 as f64 * scale_factor_y)+35.0) as i32;
                    min_x = min_x2 as i32;
                    max_x = max_x2 as i32;
                    min_y = min_y2 as i32;
                    max_y = max_y2 as i32;
                } else {
                    min_y = (min_y2 as f64 * scale_factor_y) as i32;
                    max_y = (max_y2 as f64 * scale_factor_y) as i32;
                    //println!("minx {} maxx {} miny {} maxy {}",min_x,max_x,min_y,max_y);
                    let image = screen.capture_area(min_x as i32, min_y as i32, (max_x - min_x) as u32, (max_y - min_y) as u32).unwrap();
                    let buffer = image.to_png(None).unwrap();
                    data.image_data = buffer;
                    // empty positions
                    data.positions = vec![];
                }

                let mut dynamic_image = load_from_memory_with_format(&data.image_data, image::ImageFormat::Png)
                    .expect("Failed to load image from memory");
                let mut window_width= 0;
                let mut window_height = 0;
                let mut cropped_annotated_image = dynamic_image.clone();
                let mut image_buf = ImageBuf::empty();
                if !data.first_screen {

                    match data.annotation {
                        Annotation::None => {
                            if min_x < 0 || min_y < 0 || (max_x - min_x) <=0 || (max_y - min_y) <=0 {
                                let rgba_image = dynamic_image.to_rgba8();
                                let buffer = ImageBuf::from_raw(
                                    rgba_image.clone().into_raw(),
                                    ImageFormat::RgbaSeparate,
                                    rgba_image.clone().width() as usize,
                                    rgba_image.clone().height() as usize,
                                );
                                let rect = druid::Screen::get_monitors()[data.monitor_index].virtual_rect();
                                ctx.window().close();
                                ctx.new_window(WindowDesc::new(Flex::column().with_child(Label::new("Cannot Crop Further, choose if save the image as it is or undo:"))
                                    .with_child(Image::new(buffer))
                                    .with_child(create_save_cancel_buttons())).set_position((rect.x0,rect.y0))
                                    .window_size(Size::new( 3.0 * dynamic_image.width() as f64 * data.scale_factor,(3.0 * dynamic_image.height() as f64 * data.scale_factor + BUTTON_HEIGHT * 4.0)))
                                    .resizable(false));
                                return;
                            }
                            cropped_annotated_image = dynamic_image.crop(
                                min_x as u32,
                                min_y as u32,
                                (max_x - min_x) as u32,
                                (max_y - min_y) as u32
                            );
                            if cropped_annotated_image.width() >= (screen.display_info.width as f64 * LIMIT_PROPORTION) as u32 || cropped_annotated_image.height() >= (screen.display_info.height as f64 * LIMIT_PROPORTION) as u32 {
                                data.scale_factor = SCALE_FACTOR;
                            } else {
                                data.scale_factor = 1.0;
                            }
                            cropped_annotated_image = cropped_annotated_image.resize((cropped_annotated_image.width() as f64 * data.scale_factor) as u32, (cropped_annotated_image.height() as f64 * data.scale_factor) as u32, FilterType::Nearest);

                            window_width = cropped_annotated_image.width();
                            window_height = cropped_annotated_image.height();

                        },
                        Annotation::Circle => {
                            // compute the center
                            let center_x = (max_x - min_x) as f64 / 2.0 + min_x as f64;
                            let center_y = (max_y - min_y) as f64 / 2.0 + min_y as f64;
                            let radius = (((max_x - min_x).pow(2) + (max_y - min_y).pow(2)) as f64).sqrt()/ 2.0;


                            let image = load_from_memory_with_format(&data.image_data, image::ImageFormat::Png)
                                .expect("Failed to load image from memory");

                            cropped_annotated_image = DynamicImage::from(draw_hollow_circle(&image, (center_x as i32, center_y as i32), radius as i32, Rgba([data.color.0,
                                data.color.1, data.color.2, data.color.3])));

                            window_width = cropped_annotated_image.width();
                            window_height = cropped_annotated_image.height();

                        },
                        Annotation::Line => {
                            // draw line
                            let image = load_from_memory_with_format(&data.image_data, image::ImageFormat::Png)
                                .expect("Failed to load image from memory");

                            println!("{:?}",data.color);
                            // draw line with first and last position, then clear the vector
                            cropped_annotated_image = DynamicImage::from(
                                draw_line_segment(&image,
                                                  (data.positions[0].0 as f32, data.positions[0].1 as f32),
                                                  (data.positions[data.positions.len()-1].0 as f32, data.positions[data.positions.len()-1].1 as f32),
                                                  Rgba([data.color.0, data.color.1, data.color.2, data.color.3])));
                            // clear the vector
                            data.positions = vec![];
                            window_width = cropped_annotated_image.width();
                            window_height = cropped_annotated_image.height();
                        },
                        Annotation::Cross => {
                            // draw cross through two lines
                            let image = load_from_memory_with_format(&data.image_data, image::ImageFormat::Png)
                                .expect("Failed to load image from memory");

                            // draw line with first and last position, then clear the vector
                            cropped_annotated_image = DynamicImage::from(
                                draw_line_segment(&image,
                                                  (data.positions[0].0 as f32, data.positions[0].1 as f32),
                                                  (data.positions[data.positions.len()-1].0 as f32, data.positions[data.positions.len()-1].1 as f32),
                                                  Rgba([data.color.0, data.color.1, data.color.2, data.color.3])));

                            // draw line with first and last position, then clear the vector
                            cropped_annotated_image = DynamicImage::from(
                                draw_line_segment(&cropped_annotated_image,
                                                  (data.positions[0].0 as f32, data.positions[data.positions.len()-1].1 as f32),
                                                  (data.positions[data.positions.len()-1].0 as f32, data.positions[0].1 as f32),
                                                  Rgba([data.color.0, data.color.1, data.color.2, data.color.3])));

                            // clear the vector
                            data.positions = vec![];

                            window_width = cropped_annotated_image.width();
                            window_height = cropped_annotated_image.height();
                        },
                        Annotation::Rectangle => {
                            // draw rectangle
                            let image = load_from_memory_with_format(&data.image_data, image::ImageFormat::Png)
                                .expect("Failed to load image from memory");

                            let rectangle = imageproc::rect::Rect::at(min_x, min_y).of_size((max_x - min_x) as u32, (max_y - min_y) as u32);
                            cropped_annotated_image = DynamicImage::from(
                                draw_hollow_rect(&image,rectangle,Rgba([data.color.0, data.color.1, data.color.2, data.color.3])));

                            window_width = cropped_annotated_image.width();
                            window_height = cropped_annotated_image.height();
                        },
                        Annotation::FreeLine => {
                            // draw free line
                            cropped_annotated_image = load_from_memory_with_format(&data.image_data, image::ImageFormat::Png)
                                .expect("Failed to load image from memory");

                            // draw line with first and last position, then clear the vector
                            for pos_index in 0..(data.positions.len()-1) {
                                cropped_annotated_image = DynamicImage::from(
                                    draw_line_segment(&cropped_annotated_image,
                                                      (data.positions[pos_index].0 as f32, data.positions[pos_index].1 as f32),
                                                      (data.positions[pos_index+1].0 as f32, data.positions[pos_index+1].1 as f32),
                                                      Rgba([data.color.0, data.color.1, data.color.2, data.color.3])));
                            }

                            // clear the vector
                            data.positions = vec![];
                            window_width = cropped_annotated_image.width();
                            window_height = cropped_annotated_image.height();
                        },
                        Annotation::Highlighter => {
                            // draw highliter
                            cropped_annotated_image = load_from_memory_with_format(&data.image_data, image::ImageFormat::Png)
                                .expect("Failed to load image from memory");
                            /*draw_line_segment(&cropped_annotated_image,
                                              (data.positions[pos_index].0 as f32, data.positions[pos_index].1 as f32),
                                              (data.positions[pos_index+1].0 as f32, data.positions[pos_index+1].1 as f32),
                                              Rgba([data.color.0, data.color.1, data.color.2, data.color.3]))*/
                            // draw highliter with first and last position, then clear the vector
                            for pos_index in 0..(data.positions.len()-1) {
                                let rectangle = imageproc::rect::Rect::at(data.positions[pos_index].0 as i32, data.positions[pos_index].1 as i32)
                                    .of_size((data.positions[pos_index+1].0 - data.positions[pos_index].0) as u32, 20);
                                cropped_annotated_image = DynamicImage::from(
                                    draw_polygon(&cropped_annotated_image,
                                                 &[imageproc::point::Point::new(data.positions[pos_index].0 as i32, data.positions[pos_index].1 as i32),
                                                     imageproc::point::Point::new((data.positions[pos_index+1].0 - data.positions[pos_index].0) as i32, 20)],
                                                 Rgba([data.color.0, data.color.1, data.color.2, data.color.3])));
                            }

                            // clear the vector
                            data.positions = vec![];
                            window_width = cropped_annotated_image.width();
                            window_height = cropped_annotated_image.height();
                        },
                        Annotation::Arrow => {

                        },
                        Annotation::Text => {
                            // draw line
                            let image = load_from_memory_with_format(&data.image_data, image::ImageFormat::Png)
                                .expect("Failed to load image from memory");

                            let font_data: &[u8] = include_bytes!("../OpenSans-Semibold.ttf");
                                //fs::read(Path::new(format!("{}{}",std::env::current_dir().unwrap().to_str().unwrap(),"\\OpenSans-Semibold.ttf").as_str())).unwrap().as_slice();
                            let font: Font<'static> = Font::try_from_bytes(font_data).unwrap();
                            // draw line with first and last position, then clear the vector
                            cropped_annotated_image = DynamicImage::from(
                                draw_text(&image,
                                          Rgba([data.color.0, data.color.1, data.color.2, data.color.3]),
                                          data.positions[0].0 as i32, data.positions[0].1 as i32,
                                          rusttype::Scale::uniform(20.0), &font, "prova"));
                            // clear the vector
                            data.positions = vec![];
                            window_width = cropped_annotated_image.width();
                            window_height = cropped_annotated_image.height();
                        },
                    }

                    // clear the position vector for cases different than line
                    data.positions = vec![];
                    let mut png_buffer = std::io::Cursor::new(Vec::new());
                    cropped_annotated_image.write_to(&mut png_buffer, image::ImageFormat::Png)
                        .expect("Failed to Save Cropped Image");
                    data.image_data = png_buffer.into_inner();

                    let rgba_image = cropped_annotated_image.to_rgba8();
                    image_buf = ImageBuf::from_raw(
                        rgba_image.clone().into_raw(),
                        ImageFormat::RgbaSeparate,
                        rgba_image.clone().width() as usize,
                        rgba_image.clone().height() as usize,
                    );

                } else {
                    if dynamic_image.width() >= (screen.display_info.width as f64 * LIMIT_PROPORTION) as u32 || dynamic_image.height() >= (screen.display_info.height as f64 * LIMIT_PROPORTION) as u32 {
                        data.scale_factor = SCALE_FACTOR;
                    } else {
                        data.scale_factor = 1.0;
                    }

                    dynamic_image = dynamic_image.resize((dynamic_image.width() as f64 * data.scale_factor) as u32, (dynamic_image.height() as f64 * data.scale_factor) as u32, FilterType::Nearest);

                    let mut png_buffer = std::io::Cursor::new(Vec::new());
                    dynamic_image.write_to(&mut png_buffer, image::ImageFormat::Png)
                        .expect("Failed to Save Cropped Image");
                    data.image_data = png_buffer.into_inner();

                    window_width = dynamic_image.width();
                    window_height = dynamic_image.height();

                    let rgba_image = dynamic_image.to_rgba8();
                    image_buf = ImageBuf::from_raw(
                        rgba_image.clone().into_raw(),
                        ImageFormat::RgbaSeparate,
                        rgba_image.clone().width() as usize,
                        rgba_image.clone().height() as usize,
                    );
                    data.first_screen = false;
                }

                let image = Image::new(image_buf);//.fill_mode(FillStrat::None);

                let image_data_clone = data.image_data.clone(); // Clone the data for use in the closure

                let clipboard_button = Button::new("Copy to Clipboard").on_click(move |_ctx, _data: &mut GrabData ,_env| {
                    // copy to the clipboard
                    /*let image = load_from_memory_with_format(&image_data_clone, image::ImageFormat::Png)
                        .expect("Failed to load image from memory");
                    //let mut image = image::open("C:\\Users\\Domenico\\Desktop\\Screen1.png").expect("failed");
                    let mut clipboard = arboard::Clipboard::new().unwrap();
                    let sizes = ((image_data_clone.len()/4) as f64).sqrt() as usize;
                    //println!("{} {}",sizes * (image.width()/image.height()) as usize,sizes * (image.height()/image.width()) as usize);
                    let img = arboard::ImageData {
                        width: sizes,
                        height: sizes,
                        bytes: Cow::from(image_data_clone.clone())
                    };
                    clipboard.set_image(img).expect("Error in Copying to the Clipboard");*/
                    Application::global().clipboard().put_formats(&mut [ClipboardFormat::new("image/png", _data.image_data.clone())]);
                }).fix_size(BUTTON_WIDTH * 2.0, BUTTON_HEIGHT);

                let rect = druid::Screen::get_monitors()[data.monitor_index].virtual_rect();

                // increase size of image for images too small
                while (window_width as f64 * data.scale_factor) <= BUTTON_WIDTH * 2.0 + 10.0 {
                    println!("Increment");
                    data.scale_factor+=(BUTTON_WIDTH * 2.0 + 10.0)/(window_width as f64);
                }

                ctx.window().close();
                ctx.new_window(
                    WindowDesc::new(
                        Flex::column().with_child(
                            Flex::column()
                                .with_child(
                                    ZStack::new(image)
                                        .with_centered_child(ScreenshotWidget)
                                )
                        ).with_child(Flex::column().with_child(create_save_cancel_buttons()).with_child(clipboard_button)
                            .with_child(create_annotation_buttons())))
                        .set_position((rect.x0,rect.y0))
                        .window_size(Size::new( window_width as f64 * data.scale_factor,(window_height as f64 * data.scale_factor + BUTTON_HEIGHT * 6.0)))
                        .resizable(true));
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &GrabData, env: &Env) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &GrabData, data: &GrabData, env: &Env) {
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &GrabData, env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, paint_ctx: &mut druid::PaintCtx, data: &GrabData, env: &druid::Env) {
    }
}

fn create_annotation_buttons() -> impl Widget<GrabData> {
    let mut ui_row1 = Flex::row();
    let mut ui_row2 = Flex::row();
    let file = File::open("settings.json").unwrap();
    let data: GrabData = from_reader(file).unwrap();
    // beizer curve ellisse

    ui_row1.add_flex_child(Button::new("✂").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::None;
    }), 1.0);
    ui_row1.add_default_spacer();
    ui_row1.add_flex_child(Button::new("◯").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::Circle;
    }),1.0);
    ui_row1.add_default_spacer();
    ui_row1.add_flex_child(Button::new("╱").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::Line;
    }), 1.0);
    ui_row1.add_default_spacer();
    ui_row1.add_flex_child(Button::new("✖").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::Cross;
    }), 1.0);
    ui_row1.add_default_spacer();
    ui_row1.add_flex_child(Button::new("▢").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::Rectangle;
    }), 1.0);
    ui_row1.add_default_spacer();


    ui_row2.add_flex_child(Button::new("〜").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::FreeLine;
    }), 1.0);
    ui_row2.add_default_spacer();
    ui_row2.add_flex_child(Button::new("⇗").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::Arrow;
    }), 1.0);
    ui_row2.add_default_spacer();
    ui_row2.add_flex_child(Button::new("A").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::Text;
    }), 1.0);
    ui_row2.add_default_spacer();
    ui_row2.add_flex_child(Button::new("💄").on_click(|ctx, data: &mut GrabData, _env| {
        data.annotation = Annotation::Highlighter;
    }), 1.0);
    ui_row2.add_default_spacer();
    ui_row2.add_flex_child(Button::from_label(Label::new("⬤").with_text_color(Color::rgba8(data.color.0,data.color.1,data.color.2,data.color.3))).on_click(|ctx, data: &mut GrabData, _env| {
        //data.annotation = Annotation::Circle;
        ctx.new_sub_window(WindowConfig::default().window_size((100.0,100.0)).show_titlebar(false), create_color_buttons(), data.clone(), _env.clone());
    }), 1.0);

    Flex::column().with_child(ui_row1).with_child(ui_row2)
}

fn create_color_buttons() -> impl Widget<GrabData> {
    // 12 colors 4 x 3
    let mut ui_col = Flex::column();
    let mut ui_row1 = Flex::row();
    let mut ui_row2 = Flex::row();
    let mut ui_row3 = Flex::row();

    // giallo verde blu viola rosso arancione rosa nero bianco marrone grigio magenta
    let orange: (u8, u8, u8, u8) = (255, 165, 0, 255);
    let pink : (u8, u8, u8, u8) = (255, 192, 203, 255);
    let brown : (u8, u8, u8, u8) = (139, 69, 19, 255);

    // giallo verde blu viola
    ui_row1.add_flex_child(Button::from_label(Label::new("⬤").with_text_color(
        Color::rgba8(
            druidColor::YELLOW.as_rgba8().0,
            druidColor::YELLOW.as_rgba8().1,
            druidColor::YELLOW.as_rgba8().2,
            druidColor::YELLOW.as_rgba8().3))).on_click(|ctx, data: &mut GrabData, _env| {

        // change the color and save
        data.color = druidColor::YELLOW.as_rgba8();
        let file = File::create("settings.json").unwrap();
        to_writer(file, data).unwrap();
        ctx.window().close();
    }), 1.0);

    ui_row1.add_flex_child(Button::from_label(Label::new("⬤").with_text_color(
        Color::rgba8(
            druidColor::GREEN.as_rgba8().0,
            druidColor::GREEN.as_rgba8().1,
            druidColor::GREEN.as_rgba8().2,
            druidColor::GREEN.as_rgba8().3))).on_click(|ctx, data: &mut GrabData, _env| {

        // change the color and save
        data.color = druidColor::GREEN.as_rgba8();
        let file = File::create("settings.json").unwrap();
        to_writer(file, data).unwrap();
        ctx.window().close();
    }), 1.0);

    ui_row1.add_flex_child(Button::from_label(Label::new("⬤").with_text_color(
        Color::rgba8(
            druidColor::BLUE.as_rgba8().0,
            druidColor::BLUE.as_rgba8().1,
            druidColor::BLUE.as_rgba8().2,
            druidColor::BLUE.as_rgba8().3))).on_click(|ctx, data: &mut GrabData, _env| {

        // change the color and save
        data.color = druidColor::BLUE.as_rgba8();
        let file = File::create("settings.json").unwrap();
        to_writer(file, data).unwrap();
        ctx.window().close();
    }), 1.0);

    ui_row1.add_flex_child(Button::from_label(Label::new("⬤").with_text_color(
        Color::rgba8(
            druidColor::PURPLE.as_rgba8().0,
            druidColor::PURPLE.as_rgba8().1,
            druidColor::PURPLE.as_rgba8().2,
            druidColor::PURPLE.as_rgba8().3))).on_click(|ctx, data: &mut GrabData, _env| {

        // change the color and save
        data.color = druidColor::PURPLE.as_rgba8();
        let file = File::create("settings.json").unwrap();
        to_writer(file, data).unwrap();
        ctx.window().close();
    }), 1.0);

    // rosso arancione rosa nero
    ui_row2.add_flex_child(Button::from_label(Label::new("⬤").with_text_color(
        Color::rgba8(
            druidColor::RED.as_rgba8().0,
            druidColor::RED.as_rgba8().1,
            druidColor::RED.as_rgba8().2,
            druidColor::RED.as_rgba8().3))).on_click(|ctx, data: &mut GrabData, _env| {

        // change the color and save
        data.color = druidColor::RED.as_rgba8();
        let file = File::create("settings.json").unwrap();
        to_writer(file, data).unwrap();
        ctx.window().close();
    }), 1.0);

    ui_row2.add_flex_child(Button::from_label(Label::new("⬤").with_text_color(
        Color::rgba8(
            orange.0,
            orange.1,
            orange.2,
            orange.3))).on_click(move |ctx, data: &mut GrabData, _env| {

        // change the color and save
        data.color = orange;
        let file = File::create("settings.json").unwrap();
        to_writer(file, data).unwrap();
        ctx.window().close();
    }), 1.0);

    ui_row2.add_flex_child(Button::from_label(Label::new("⬤").with_text_color(
        Color::rgba8(
            pink.0,
            pink.1,
            pink.2,
            pink.3))).on_click(move |ctx, data: &mut GrabData, _env| {

        // change the color and save
        data.color = pink;
        let file = File::create("settings.json").unwrap();
        to_writer(file, data).unwrap();
        ctx.window().close();
    }), 1.0);

    ui_row2.add_flex_child(Button::from_label(Label::new("⬤").with_text_color(
        Color::rgba8(
            druidColor::BLACK.as_rgba8().0,
            druidColor::BLACK.as_rgba8().1,
            druidColor::BLACK.as_rgba8().2,
            druidColor::BLACK.as_rgba8().3))).on_click(|ctx, data: &mut GrabData, _env| {

        // change the color and save
        data.color = druidColor::BLACK.as_rgba8();
        let file = File::create("settings.json").unwrap();
        to_writer(file, data).unwrap();
        ctx.window().close();
    }), 1.0);

    // bianco marrone grigio magenta
    ui_row3.add_flex_child(Button::from_label(Label::new("⬤").with_text_color(
        Color::rgba8(
            druidColor::WHITE.as_rgba8().0,
            druidColor::WHITE.as_rgba8().1,
            druidColor::WHITE.as_rgba8().2,
            druidColor::WHITE.as_rgba8().3))).on_click(|ctx, data: &mut GrabData, _env| {

        // change the color and save
        data.color = druidColor::WHITE.as_rgba8();
        let file = File::create("settings.json").unwrap();
        to_writer(file, data).unwrap();
        ctx.window().close();
    }), 1.0);

    ui_row3.add_flex_child(Button::from_label(Label::new("⬤").with_text_color(
        Color::rgba8(
            brown.0,
            brown.1,
            brown.2,
            brown.3))).on_click(move |ctx, data: &mut GrabData, _env| {

        // change the color and save
        data.color = brown;
        let file = File::create("settings.json").unwrap();
        to_writer(file, data).unwrap();
        ctx.window().close();
    }), 1.0);

    ui_row3.add_flex_child(Button::from_label(Label::new("⬤").with_text_color(
        Color::rgba8(
            druidColor::GRAY.as_rgba8().0,
            druidColor::GRAY.as_rgba8().1,
            druidColor::GRAY.as_rgba8().2,
            druidColor::GRAY.as_rgba8().3))).on_click(|ctx, data: &mut GrabData, _env| {

        // change the color and save
        data.color = druidColor::GRAY.as_rgba8();
        let file = File::create("settings.json").unwrap();
        to_writer(file, data).unwrap();
        ctx.window().close();
    }), 1.0);

    ui_row3.add_flex_child(Button::from_label(Label::new("⬤").with_text_color(
        Color::rgba8(
            druidColor::FUCHSIA.as_rgba8().0,
            druidColor::FUCHSIA.as_rgba8().1,
            druidColor::FUCHSIA.as_rgba8().2,
            druidColor::FUCHSIA.as_rgba8().3))).on_click(|ctx, data: &mut GrabData, _env| {

        // change the color and save
        data.color = druidColor::FUCHSIA.as_rgba8();
        let file = File::create("settings.json").unwrap();
        to_writer(file, data).unwrap();
        ctx.window().close();
    }), 1.0);

    ui_col.add_flex_child(ui_row1, 1.0);
    ui_col.add_default_spacer();
    ui_col.add_flex_child(ui_row2, 1.0);
    ui_col.add_default_spacer();
    ui_col.add_flex_child(ui_row3, 1.0);
    ui_col.add_default_spacer();

    ui_col
}