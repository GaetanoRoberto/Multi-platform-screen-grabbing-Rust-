use std::borrow::Cow;
use std::cmp::min;
use std::fs;
use std::fs::create_dir_all;
use druid::{Application, BoxConstraints, Clipboard, ClipboardFormat, Color, commands, Cursor, Env, Event, EventCtx, FormatId, ImageBuf, LayoutCtx, LifeCycle, LifeCycleCtx, MouseButton, Point, Rect, Screen, Size, UnitPoint, UpdateCtx, Vec2, Widget, WidgetExt, WindowConfig, WindowDesc};
use druid::piet::{ImageFormat, InterpolationMode};
use druid::piet::PaintBrush::Fixed;
use druid::platform_menus::mac::file::print;
use druid::widget::{ZStack, Button, Container, Flex, Image, SizedBox, FillStrat};
use image::{DynamicImage, EncodableLayout, ImageBuffer, load_from_memory_with_format, Rgba};
use image::imageops::FilterType;
use crate::{main_gui_building::build_ui, constants, GrabData};
use constants::{BUTTON_HEIGHT,BUTTON_WIDTH,LIMIT_PROPORTION,SCALE_FACTOR};
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

                // empty positions
                data.positions = vec![];

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
                }

                let mut dynamic_image = load_from_memory_with_format(&data.image_data, image::ImageFormat::Png)
                    .expect("Failed to load image from memory");
                let window_width;
                let window_height;
                let cropped_image;
                let image_buf;
                if !data.first_screen {

                    //let image_rect_cropped = Rect::new(min_x as f64,min_y as f64,max_x as f64,max_y as f64);
                    /*let tras_x = (ctx.window().get_size().width - dynamic_image.width() as f64)/2.0;
                    let tras_y = ctx.window().get_size().height - dynamic_image.height() as f64 - 200.0;
                    min_x = (min_x2 - tras_x) as i32;
                    max_x = (max_x2 - tras_x) as i32;
                    min_y = (min_y2 - tras_y) as i32;
                    max_y = (max_y2 - tras_y) as i32;
                    println!("{}",ctx.window().get_size().width - dynamic_image.width() as f64);
                    // 200 ok (sotto ho messo + 200)
                    println!("{}",ctx.window().get_size().height - dynamic_image.height() as f64);
                    println!("{} {} {} {}",min_x,min_y,(max_x - min_x),(max_y - min_y));*/
                    cropped_image = dynamic_image.crop(
                        min_x as u32,
                        min_y as u32,
                        (max_x - min_x) as u32,
                        (max_y - min_y) as u32
                    );
                    if cropped_image.width() >= (screen.display_info.width as f64 * LIMIT_PROPORTION) as u32 || cropped_image.height() >= (screen.display_info.height as f64 * LIMIT_PROPORTION) as u32 {
                        data.scale_factor = SCALE_FACTOR;
                    } else {
                        data.scale_factor = 1.0;
                    }
                    cropped_image.resize((cropped_image.width() as f64 * data.scale_factor) as u32, (cropped_image.height() as f64 * data.scale_factor) as u32, FilterType::Nearest);

                    window_width = cropped_image.width();
                    window_height = cropped_image.height();

                    let mut png_buffer = std::io::Cursor::new(Vec::new());
                    cropped_image.write_to(&mut png_buffer, image::ImageFormat::Png)
                        .expect("Failed to Save Cropped Image");
                    data.image_data = png_buffer.into_inner();

                    let rgba_image = cropped_image.to_rgba8();
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

                    dynamic_image.resize((dynamic_image.width() as f64 * data.scale_factor) as u32, (dynamic_image.height() as f64 * data.scale_factor) as u32, FilterType::Nearest);

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

                let mut rect = druid::Screen::get_monitors()[data.monitor_index].virtual_rect();

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
                        ).with_child(Flex::column().with_child(Flex::row().with_child(save_button).with_child(cancel_button)).with_child(clipboard_button)))
                        .set_position((rect.x0,rect.y0))
                        .window_size(Size::new( window_width as f64 * data.scale_factor,(window_height as f64 * data.scale_factor + BUTTON_HEIGHT * 4.0)))
                        .resizable(false));
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