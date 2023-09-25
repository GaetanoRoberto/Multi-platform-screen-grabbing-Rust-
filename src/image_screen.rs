use std::borrow::Cow;
use std::cmp::{max, min};
use std::fs;
use std::fs::{create_dir_all, File};
use std::io::Read;
use std::path::Path;
use druid::{Application, BoxConstraints, Clipboard, ClipboardFormat, Color, commands, Cursor, Env, Event, EventCtx, FormatId, ImageBuf, LayoutCtx, LifeCycle, LifeCycleCtx, MouseButton, Point, Rect, RenderContext, Scale, Screen, Size, UnitPoint, UpdateCtx, Vec2, Widget, WidgetExt, WindowConfig, WindowDesc};
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
use crate::main_gui_building::{create_annotation_buttons, create_save_cancel_buttons};
use rusttype::Font;
use druid::{kurbo::Line, piet::StrokeStyle, kurbo::Shape, PaintCtx};
use druid::Handled::No;
use crate::constants::BORDER_WIDTH;

//use ease::{Arrow, EaseType, Tweener};

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
            ctx.request_paint();
        }

        if let Event::MouseUp(_) = event {
            data.press = false;
            ctx.request_paint();
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

                let (min_x2,min_y2,max_x2,max_y2) = make_rectangle_from_points(data).unwrap();

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
                    let image = screen.capture_area(min_x + BORDER_WIDTH as i32, min_y + BORDER_WIDTH as i32, (max_x - (min_x + 2*BORDER_WIDTH as i32)) as u32, (max_y - (min_y + 2*BORDER_WIDTH as i32)) as u32).unwrap();
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
                    let image = load_from_memory_with_format(&image_data_clone, image::ImageFormat::Png).unwrap().to_rgba8();
                    let  mut clipboard = arboard::Clipboard::new().unwrap();

                    let img = arboard::ImageData {
                        width: image.width() as usize,
                        height: image.height() as usize,
                        bytes: Cow::from(image.as_bytes())
                    };
                    clipboard.set_image(img).unwrap();

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
        let result = make_rectangle_from_points(data);
        match result {
            Some((x0,y0,x1,y1)) => {
                let mut rect = Rect::new(x0, y0, x1, y1);
                let mut border_color;

                rect = Rect::new(x0 * data.scale_factor, y0 * data.scale_factor, x1 * data.scale_factor, y1 * data.scale_factor);
                border_color = Color::rgb(255.0, 255.0, 255.0); // White border color

                // Create a shape representing the rectangle
                let rect_shape = Rect::from_origin_size(rect.origin(), rect.size());

                // Draw the border of the rectangle
                paint_ctx.stroke(rect_shape, &border_color, BORDER_WIDTH);
                //paint_ctx.fill(rect_shape, &Color::rgba8(255, 255, 255, 0));
            }
            None => { }
        }
    }
}

fn make_rectangle_from_points(data: &GrabData ) -> Option<(f64,f64,f64,f64)> {
    if data.positions.is_empty() {
        return None;
    }
    let (mut min_x,mut max_y) = (0.0,0.0);
    let (mut max_x,mut min_y) = (0.0,0.0);
    let (p1x,p1y) = data.positions[0];
    let (p2x,p2y) = data.positions[data.positions.len() - 1];

    if p1x < p2x && p1y < p2y {
        // p1 smaller than p2
        min_x = p1x;
        min_y = p1y;
        max_x = p2x;
        max_y = p2y;
    } else if p1x > p2x && p1y > p2y {
        // p2 smaller than p1
        min_x = p2x;
        min_y = p2y;
        max_x = p1x;
        max_y = p1y;
    } else if p1x < p2x && p1y > p2y {
        // partenza in basso a sx
        min_x = p1x;
        min_y = p2y;
        max_x = p2x;
        max_y = p1y;
    } else if p1x > p2x && p1y < p2y {
        // partenza in alto a dx
        min_x = p2x;
        min_y = p1y;
        max_x = p1x;
        max_y = p2y;
    }

    Some((min_x,min_y,max_x,max_y))
}

