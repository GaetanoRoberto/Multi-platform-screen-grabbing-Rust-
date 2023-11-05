use std::borrow::Cow;
use std::cmp::{max, min};
use std::fs;
use std::fs::{create_dir_all, File};
use std::io::Read;
use std::path::Path;
use druid::{Application, BoxConstraints, Clipboard, ClipboardFormat, Color, commands, Cursor, Env, Event, EventCtx, FormatId, ImageBuf, LayoutCtx, LifeCycle, LifeCycleCtx, MouseButton, MouseEvent, Point, Rect, RenderContext, Scale, Screen, Size, UnitPoint, UpdateCtx, Vec2, Widget, WidgetExt, WindowConfig, WindowDesc};
use druid::piet::{ImageFormat, InterpolationMode};
use druid::piet::PaintBrush::Fixed;
use druid::platform_menus::mac::file::print;
use druid::widget::{ZStack, Button, Container, Flex, Image, SizedBox, FillStrat, Label, ClipBox, Controller};
use image::{DynamicImage, EncodableLayout, ImageBuffer, load_from_memory_with_format, Rgba};
use image::imageops::{FilterType, overlay};
use imageproc::drawing::{Canvas, draw_cross, draw_filled_rect, draw_hollow_circle, draw_hollow_circle_mut, draw_hollow_rect, draw_line_segment, draw_polygon, draw_text};
use serde_json::{from_reader, to_writer};
use crate::{main_gui_building::build_ui, constants, GrabData, Annotation};
use constants::{BUTTON_HEIGHT,BUTTON_WIDTH,SCALE_FACTOR};
use crate::main_gui_building::{create_annotation_buttons, create_edit_window, create_edit_window_widgets, create_save_cancel_clipboard_buttons, create_selection_window};
use rusttype::Font;
use druid::{kurbo::Line, piet::StrokeStyle, kurbo::Shape, PaintCtx};
use druid::Handled::No;
use crate::constants::{APP_NAME, BORDER_WIDTH, TRANSPARENCY};
use std::f64::consts::PI;
use druid::kurbo::{BezPath, Circle};
use druid_widget_nursery::stack_tooltip::tooltip_state_derived_lenses::data;
use image::codecs::pnm::ArbitraryTuplType::RGBAlpha;
use serde_json::error::Category::Data;
use crate::grab_data_derived_lenses::highlighter_width;
use crate::utilities::{compute_offsets, make_rectangle_from_points, load_image, compute_circle_center_radius, compute_arrow_points, image_to_buffer, compute_highlighter_points, resize_image, screen_all};

pub struct ScreenshotWidget;

impl Widget<GrabData> for ScreenshotWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut GrabData, _env: &Env) {
        let mut min_x = 0;
        let mut min_y = 0;
        let mut max_x = 0;
        let mut max_y = 0;

        if let Event::MouseDown(mouse_event) = event {
            if mouse_event.button.is_left() {
                compute_offsets(ctx, data);
                data.press = true;
            }
            //if annotation text, simply take the point where the mouse is pressed (take no point when mouse moves)
            if data.annotation == Annotation::Text {
                data.positions.push((mouse_event.window_pos.x,mouse_event.window_pos.y));
            }
        }
        /*if let Event::WindowConnected = event {
            data.scale_factor = ctx.window().get_size().height / ctx.window().get_size().width;
        }
        if let Event::WindowSize(wsize) = event {
            data.scale_factor = wsize.aspect_ratio();
        }*/
        if let Event::MouseMove(mouse_event) = event {
            if data.annotation == Annotation::Text {
                ctx.set_cursor(&Cursor::IBeam);
            } else {
                ctx.set_cursor(&Cursor::Crosshair);
                if data.press {
                    data.positions.push((mouse_event.window_pos.x,mouse_event.window_pos.y));
                }

            }
            ctx.request_paint();
        }

        if let Event::MouseUp(_) = event {
            data.press = false;

            if !data.positions.is_empty() {
                let screen = screenshots::Screen::all().unwrap()[data.monitor_index];

                let (min_x2,min_y2,max_x2,max_y2) = make_rectangle_from_points(data).unwrap();

                if !data.first_screen {
                    min_x = min_x2 as i32;
                    max_x = max_x2 as i32;
                    min_y = min_y2 as i32;
                    max_y = max_y2 as i32;
                } else {
                    let scale_factor_x = ctx.scale().x();
                    let scale_factor_y = ctx.scale().y();
                    min_x = (min_x2 * scale_factor_x) as i32;
                    max_x = (max_x2 * scale_factor_x) as i32;
                    min_y = (min_y2 * scale_factor_y) as i32;
                    max_y = (max_y2 * scale_factor_y) as i32;

                    /*let image = screen.capture_area(min_x + BORDER_WIDTH as i32, min_y + BORDER_WIDTH as i32, (max_x - (min_x + 2*BORDER_WIDTH as i32)) as u32, (max_y - (min_y + 2*BORDER_WIDTH as i32)) as u32).unwrap();
                    let buffer = image.to_png(None).unwrap();

                    data.image_data_old = buffer;*/
                    screen_all(min_x,min_y,max_x,max_y,data);
                    // empty positions
                    data.positions = vec![];
                }

                let mut dynamic_image = load_image(data);
                let mut window_width= 0;
                let mut window_height = 0;
                let mut cropped_annotated_image = dynamic_image.clone();
                if !data.first_screen {

                    match data.annotation {
                        Annotation::None => {
                            if min_x < 0 || min_y < 0 || ((max_x - min_x) as f64 * data.scale_factors.0) as u32 <= 0
                                || ((max_y - min_y) as f64 * data.scale_factors.1) as u32 <=0 {
                                let rgba_image = dynamic_image.to_rgba8();
                                let buffer = ImageBuf::from_raw(
                                    rgba_image.clone().into_raw(),
                                    ImageFormat::RgbaSeparate,
                                    rgba_image.clone().width() as usize,
                                    rgba_image.clone().height() as usize,
                                );
                                let rect = druid::Screen::get_monitors()[data.monitor_index].virtual_rect();
                                let (image_width,image_height) = resize_image(dynamic_image,data);

                                ctx.window().close();
                                ctx.new_window(WindowDesc::new(Flex::column().with_child(Label::new("Cannot Crop: Image too Small. \nChoose if save the image as it is or undo:"))
                                    .with_child(SizedBox::new(Image::new(buffer)).width(image_width).height(image_height))
                                    .with_child(create_save_cancel_clipboard_buttons())).title(APP_NAME).set_position((rect.x0,rect.y0))
                                    .with_min_size(Size::new(5.0 * BUTTON_WIDTH,3.0* BUTTON_HEIGHT))
                                    .window_size(Size::new( image_width,(image_height + BUTTON_HEIGHT * 5.0)))
                                    .resizable(false));

                                data.positions = vec![];
                                return;
                            }

                            cropped_annotated_image = dynamic_image.crop(
                                ((min_x as f64 - data.offsets.0) * data.scale_factors.0) as u32,
                                ((min_y as f64 - data.offsets.1) * data.scale_factors.1) as u32,
                                (((max_x as f64- data.offsets.0) - (min_x as f64 - data.offsets.0)) * data.scale_factors.0) as u32,
                                (((max_y as f64- data.offsets.1) - (min_y as f64 - data.offsets.1)) * data.scale_factors.1) as u32
                            );

                            /*if cropped_annotated_image.width() >= (screen.display_info.width as f64 * LIMIT_PROPORTION) as u32 || cropped_annotated_image.height() >= (screen.display_info.height as f64 * LIMIT_PROPORTION) as u32 {
                                data.scale_factor = SCALE_FACTOR;
                            } else {
                                data.scale_factor = 1.0;
                            }*/
                            // cropped_annotated_image = cropped_annotated_image.resize((cropped_annotated_image.width() as f64 * data.scale_factor) as u32, (cropped_annotated_image.height() as f64 * data.scale_factor) as u32, FilterType::Nearest);

                        },
                        Annotation::Circle => {
                            // compute the center and the radius
                            let (mut center_x, mut center_y) = compute_circle_center_radius(data, min_x, min_y, max_x, max_y);
                            let image = load_image(data);
                            let radius = (((data.scale_factors.0 * (max_x - min_x) as f64).powi(2) + (data.scale_factors.1 * (max_y - min_y) as f64).powi(2)) as f64).sqrt()/ 2.0;

                            cropped_annotated_image = DynamicImage::from(draw_hollow_circle(&image, ((center_x * data.scale_factors.0) as i32, (center_y * data.scale_factors.1) as i32), radius as i32, Rgba([data.color.0,
                                data.color.1, data.color.2, data.color.3])));

                        },
                        Annotation::Line => {
                            // draw line
                            let image = load_image(data);

                            // draw line with first and last position, then clear the vector
                            let p0 = (data.positions[0].0 , data.positions[0].1 );
                            let p1 = (data.positions[data.positions.len()-1].0 ,
                                      data.positions[data.positions.len()-1].1 );

                            cropped_annotated_image = DynamicImage::from(
                                draw_line_segment(&image,
                                                  (((p0.0 - data.offsets.0) * data.scale_factors.0) as f32, ((p0.1 - data.offsets.1) * data.scale_factors.1) as f32),
                                                  (((p1.0 - data.offsets.0) * data.scale_factors.0) as f32, ((p1.1 - data.offsets.1) * data.scale_factors.1) as f32),
                                                  Rgba([data.color.0, data.color.1, data.color.2, data.color.3])));

                        },
                        Annotation::Cross => {
                            // draw cross through two lines
                            let image = load_image(data);

                            let line0_p0 = (data.positions[0].0 - data.offsets.0, data.positions[0].1 - data.offsets.1);
                            let line0_p1 = (data.positions[data.positions.len()-1].0 - data.offsets.0,
                                            data.positions[data.positions.len()-1].1 - data.offsets.1);

                            // draw line with first and last position, then clear the vector
                            cropped_annotated_image = DynamicImage::from(
                                draw_line_segment(&image,
                                                  ((line0_p0.0 * data.scale_factors.0) as f32, (line0_p0.1 * data.scale_factors.1) as f32),
                                                  ((line0_p1.0 * data.scale_factors.0) as f32, (line0_p1.1 * data.scale_factors.1) as f32),
                                                  Rgba([data.color.0, data.color.1, data.color.2, data.color.3])));

                            let line1_p0 = (data.positions[0].0 - data.offsets.0, data.positions[data.positions.len()-1].1 - data.offsets.1);
                            let line1_p1 = (data.positions[data.positions.len()-1].0 - data.offsets.0,data.positions[0].1 - data.offsets.1);

                            // draw line with first and last position, then clear the vector
                            cropped_annotated_image = DynamicImage::from(
                                draw_line_segment(&cropped_annotated_image,
                                                  ((line1_p0.0 * data.scale_factors.0) as f32, (line1_p0.1 * data.scale_factors.1) as f32),
                                                  ((line1_p1.0 * data.scale_factors.0) as f32, (line1_p1.1 * data.scale_factors.1) as f32),
                                                  Rgba([data.color.0, data.color.1, data.color.2, data.color.3])));

                        },
                        Annotation::Rectangle => {
                            // draw rectangle
                            let image = load_image(data);

                            let rectangle = imageproc::rect::Rect::at(((min_x as f64 - data.offsets.0) * data.scale_factors.0) as i32, ((min_y as f64 - data.offsets.1) * data.scale_factors.1) as i32).of_size((((max_x as f64 - data.offsets.0) - (min_x as f64 - data.offsets.0)) * data.scale_factors.0 ) as u32,( ((max_y as f64 - data.offsets.1) - (min_y as f64 - data.offsets.1)) * data.scale_factors.1) as u32);
                            cropped_annotated_image = DynamicImage::from(
                                draw_hollow_rect(&image,rectangle,Rgba([data.color.0, data.color.1, data.color.2, data.color.3])));

                        },
                        Annotation::FreeLine => {
                            // draw free line
                            cropped_annotated_image = load_image(data);

                            // draw line with first and last position, then clear the vector
                            for pos_index in 0..(data.positions.len()-1) {
                                let line_p0 = (data.positions[pos_index].0, data.positions[pos_index].1);
                                let line_p1 = (data.positions[pos_index+1].0, data.positions[pos_index+1].1);

                                cropped_annotated_image = DynamicImage::from(
                                    draw_line_segment(&cropped_annotated_image,
                                                       (((line_p0.0 - data.offsets.0) * data.scale_factors.0) as f32, ((line_p0.1 - data.offsets.1) * data.scale_factors.1) as f32),
                                                      (((line_p1.0 - data.offsets.0) * data.scale_factors.0) as f32, ((line_p1.1 - data.offsets.1) * data.scale_factors.1) as f32),
                                                      Rgba([data.color.0, data.color.1, data.color.2, data.color.3])));
                            }

                        },
                        Annotation::Highlighter => {
                            // draw highliter
                            cropped_annotated_image = load_image(data);
                            let mut transparent_image =  ImageBuffer::from_pixel(
                                cropped_annotated_image.width(),
                                cropped_annotated_image.height(),
                                Rgba([0, 0, 0, 0]));

                            // get the highliter points
                            let result = compute_highlighter_points(data);

                            match result {
                                Some((rect_point1,rect_point2,rect_point3,rect_point4)) => {
                                    let poly = &[imageproc::point::Point::new((rect_point1.x * data.scale_factors.0) as i32,(rect_point1.y * data.scale_factors.1) as i32),
                                        imageproc::point::Point::new((rect_point2.x * data.scale_factors.0) as i32,(rect_point2.y * data.scale_factors.1) as i32),
                                        imageproc::point::Point::new((rect_point3.x * data.scale_factors.0) as i32,(rect_point3.y * data.scale_factors.1) as i32),
                                        imageproc::point::Point::new((rect_point4.x * data.scale_factors.0) as i32,(rect_point4.y * data.scale_factors.1) as i32)];

                                    transparent_image = draw_polygon(&transparent_image, poly, Rgba([data.color.0, data.color.1, data.color.2, TRANSPARENCY]));

                                    overlay(&mut cropped_annotated_image, &transparent_image, 0, 0);
                                }
                                None => {}
                            }

                        },
                        Annotation::Arrow => {
                            let image = load_image(data);

                            let result = compute_arrow_points(data);
                            match result {
                                Some(((main_line_p0,main_line_p1),(arrow_l0_p0, arrow_l0_p1),(arrow_l1_p0, arrow_l1_p1))) => {
                                    // draw line of arrow
                                    cropped_annotated_image = DynamicImage::from(
                                        draw_line_segment(&image,
                                                          ((main_line_p0.x * data.scale_factors.0) as f32, (main_line_p0.y * data.scale_factors.1) as f32),
                                                          ((main_line_p1.x * data.scale_factors.0) as f32, (main_line_p1.y * data.scale_factors.1) as f32),
                                                          Rgba([data.color.0, data.color.1, data.color.2, data.color.3])));

                                    // segmento 1 punta freccia
                                    cropped_annotated_image = DynamicImage::from(
                                        draw_line_segment(&cropped_annotated_image,
                                                          ((arrow_l0_p0.x * data.scale_factors.0) as f32, (arrow_l0_p0.y * data.scale_factors.1) as f32),
                                                          ((arrow_l0_p1.x * data.scale_factors.0) as f32, (arrow_l0_p1.y * data.scale_factors.1) as f32),
                                                          Rgba([data.color.0, data.color.1, data.color.2, data.color.3])));
                                    // segmento 2 punta freccia
                                    cropped_annotated_image = DynamicImage::from(
                                        draw_line_segment(&cropped_annotated_image,
                                                          ((arrow_l1_p0.x * data.scale_factors.0) as f32, (arrow_l1_p0.y * data.scale_factors.1) as f32),
                                                          ((arrow_l1_p1.x * data.scale_factors.0) as f32, (arrow_l1_p1.y * data.scale_factors.1) as f32),
                                                          Rgba([data.color.0, data.color.1, data.color.2, data.color.3])));

                                }
                                None => {}
                            }
                        },
                        Annotation::Text => {
                            // done in add_text button handler in main_gui_building
                        },
                    }

                    window_width = cropped_annotated_image.width();
                    window_height = cropped_annotated_image.height();

                    if data.annotation != Annotation::Text {
                        // clear the position
                        data.positions = vec![];
                        //data.annotation = Annotation::None;
                        // save the modified version of the image
                        data.image_data_new = image_to_buffer(cropped_annotated_image);
                    }

                } else {
                    /*if dynamic_image.width() >= (screen.display_info.width as f64 * LIMIT_PROPORTION) as u32 || dynamic_image.height() >= (screen.display_info.height as f64 * LIMIT_PROPORTION) as u32 {
                        data.scale_factor = SCALE_FACTOR;
                    } else {
                        data.scale_factor = 1.0;
                    }*/

                    // dynamic_image = dynamic_image.resize((dynamic_image.width() as f64 * data.scale_factor) as u32, (dynamic_image.height() as f64 * data.scale_factor) as u32, FilterType::Nearest);


                    window_width = dynamic_image.width();
                    window_height = dynamic_image.height();

                    data.image_data_old = image_to_buffer(dynamic_image);

                    data.first_screen = false;
                }

                if data.annotation != Annotation::Text {
                    if data.image_data_new.is_empty() {
                        create_selection_window(ctx,data);
                    } else {
                        create_edit_window(ctx,data);
                    }
                }

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

    fn paint(&mut self, paint_ctx: &mut druid::PaintCtx, data: & GrabData, env: &druid::Env) {
        // border color of the current selected color for all the paintings except the rectangle preview
        let mut border_color = Color::rgb8(data.color.0, data.color.1, data.color.2); // White border color

        match data.annotation {
            Annotation::None | Annotation::Rectangle => {
                let result = make_rectangle_from_points(data);
                match result {
                    Some((x0,y0,x1,y1)) => {



                        // Create a shape representing the rectangle in the widget's coordinate system
                        let rect_shape = Rect::new(x0 - data.offsets.0, y0 - data.offsets.1,
                                                   x1 - data.offsets.0, y1 - data.offsets.1);
                        if data.annotation == Annotation::None {
                            // override in white color, only for selection other cases the selected color at the beginning
                            border_color = Color::rgb8(255, 255, 255); // White border color
                        }
                        paint_ctx.stroke(rect_shape, &border_color, BORDER_WIDTH);
                    }
                    None => { }
                }
            }
            Annotation::Circle => {
                let result = make_rectangle_from_points(data);
                match result {
                    Some((min_x,min_y,max_x,max_y)) => {
                        // compute the center and the radius
                        let (center_x,center_y) = compute_circle_center_radius(data,min_x as i32, min_y as i32,max_x as i32,max_y as i32);
                        let radius = ((((max_x - min_x)).powi(2) + ((max_y - min_y)).powi(2))).sqrt()/ 2.0;

                        // Create a shape representing the circle in the widget's coordinate system
                        let circle_shape = Circle::new((center_x,center_y),radius);

                        paint_ctx.stroke(circle_shape, &border_color, BORDER_WIDTH);
                    }
                    None => {}
                }
            }
            Annotation::Line => {
                if !data.positions.is_empty() {
                    let p0 = (data.positions[0].0 - data.offsets.0, data.positions[0].1 - data.offsets.1);
                    let p1 = (data.positions[data.positions.len()-1].0 - data.offsets.0,
                              data.positions[data.positions.len()-1].1 - data.offsets.1);

                    let line_shape = Line::new(p0,p1);

                    // Create a line in the widget's coordinate system
                    paint_ctx.stroke(line_shape, &border_color, BORDER_WIDTH);
                }
            }
            Annotation::Cross => {
                if !data.positions.is_empty() {
                    let line0_p0 = (data.positions[0].0 - data.offsets.0, data.positions[0].1 - data.offsets.1);
                    let line0_p1 = (data.positions[data.positions.len()-1].0 - data.offsets.0,
                                    data.positions[data.positions.len()-1].1 - data.offsets.1);

                    let line0_shape = Line::new(line0_p0,line0_p1);

                    let line1_p0 = (data.positions[0].0 - data.offsets.0, data.positions[data.positions.len()-1].1 - data.offsets.1);
                    let line1_p1 = (data.positions[data.positions.len()-1].0 - data.offsets.0,data.positions[0].1 - data.offsets.1);

                    let line1_shape = Line::new(line1_p0,line1_p1);

                    // Create a cross in the widget's coordinate system
                    paint_ctx.stroke(line0_shape, &border_color, BORDER_WIDTH);
                    paint_ctx.stroke(line1_shape, &border_color, BORDER_WIDTH);
                }
            }
            Annotation::FreeLine => {
                if !data.positions.is_empty() {
                    // draw line with first and last position, then clear the vector
                    for pos_index in 0..(data.positions.len()-1) {
                        let line_p0 = (data.positions[pos_index].0 - data.offsets.0, data.positions[pos_index].1 - data.offsets.1);
                        let line_p1 = (data.positions[pos_index+1].0 - data.offsets.0, data.positions[pos_index+1].1 - data.offsets.1);

                        let line_shape = Line::new(line_p0,line_p1);

                        // Create a line in the widget's coordinate system
                        paint_ctx.stroke(line_shape, &border_color, BORDER_WIDTH);
                    }
                }
            }
            Annotation::Highlighter => {
                let result = compute_highlighter_points(data);
                match result {
                    Some((rect_point1,rect_point2,rect_point3,rect_point4)) => {
                        let mut path = BezPath::new();
                        path.move_to(rect_point1);
                        path.line_to(rect_point2);
                        path.line_to(rect_point3);
                        path.line_to(rect_point4);

                        border_color = Color::rgba8(data.color.0, data.color.1, data.color.2, TRANSPARENCY);

                        paint_ctx.fill(path, &border_color);
                    }
                    None => {}
                }
            }
            Annotation::Arrow => {
                let result = compute_arrow_points(data);
                match result {
                    Some(((main_line_p0,main_line_p1),(arrow_l0_p0, arrow_l0_p1),(arrow_l1_p0, arrow_l1_p1))) => {
                        let line_shape = Line::new(main_line_p0,main_line_p1);
                        // Create a line in the widget's coordinate system
                        paint_ctx.stroke(line_shape, &border_color, BORDER_WIDTH);

                        // segmento 1 punta freccia
                        let punta1_shape = Line::new(arrow_l0_p0, arrow_l0_p1);
                        // Create a line in the widget's coordinate system
                        paint_ctx.stroke(punta1_shape, &border_color, BORDER_WIDTH);

                        // segmento 2 punta freccia
                        let punta2_shape = Line::new(arrow_l1_p0, arrow_l1_p1);
                        // Create a line in the widget's coordinate system
                        paint_ctx.stroke(punta2_shape, &border_color, BORDER_WIDTH);
                    }
                    None => {}
                }
            }
            Annotation::Text => {
                if !data.positions.is_empty() {
                    // take the only point to draw the text line from it
                    // the last point if we click many times, so len-1
                    let (min_x,min_y) = (data.positions[data.positions.len()-1].0 - data.offsets.0,
                                         data.positions[data.positions.len()-1].1 - data.offsets.1);
                    let line_shape = Line::new((min_x,min_y),(min_x, min_y + data.text_size));

                    paint_ctx.stroke(line_shape, &border_color, BORDER_WIDTH);
                }
            }
        }
    }
}

