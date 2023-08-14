use std::fs;
use std::fs::create_dir_all;
use druid::{BoxConstraints, Color, commands, Cursor, Env, Event, EventCtx, ImageBuf, LayoutCtx, LifeCycle, LifeCycleCtx, MouseButton, Screen, Size, UnitPoint, UpdateCtx, Vec2, Widget, WidgetExt, WindowDesc};
use druid::piet::{ImageFormat, InterpolationMode};
use druid::piet::PaintBrush::Fixed;
use druid::platform_menus::mac::file::print;
use druid::widget::{ZStack, Button, Container, Flex, Image, SizedBox};
use image::load_from_memory_with_format;
use crate::GrabData;

pub struct ScreenshotWidget;

impl Widget<GrabData> for ScreenshotWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut GrabData, _env: &Env) {
        let mut min_x = 0;
        let mut min_y = 0;
        let mut max_x = 0;
        let mut max_y = 0;

        if let Event::MouseDown(_) = event {
            data.press = true;
            //ctx.set_cursor(&Cursor::Crosshair);
        }

        if let Event::MouseMove(mouse_event) = event {
            ctx.set_cursor(&Cursor::Crosshair);
            if data.press {
                data.positions.push((mouse_event.window_pos.x,mouse_event.window_pos.y));
            }
        }

        if let Event::MouseUp(_) = event {
            data.press = false;
            //println!("{:?}",data.positions);
            if !data.positions.is_empty() {
                let screen = screenshots::Screen::all().unwrap()[data.monitor_index];

                /*let (min_x2, min_y2) = data.positions.iter().cloned().min_by(|(x1, y1), (x2, y2)| {
                    x1.partial_cmp(x2).unwrap()
                }).unwrap();

                let (max_x2, max_y2) = data.positions.iter().cloned().max_by(|(x1, y1), (x2, y2)| {
                    x1.partial_cmp(x2).unwrap()
                }).unwrap();*/

                let (min_x2, max_y2) = data.positions.iter().cloned().fold(
                    (f64::INFINITY, f64::NEG_INFINITY),
                    |(min_x, max_y), (x, y)| (min_x.min(x), max_y.max(y)),
                );

                let (max_x2, min_y2) = data.positions.iter().cloned().fold(
                    (f64::NEG_INFINITY, f64::INFINITY),
                    |(max_x, min_y), (x, y)| (max_x.max(x), min_y.min(y)),
                );

                // empty positions
                data.positions = vec![];

                let scale_factor_x = ctx.scale().x();
                let scale_factor_y = ctx.scale().y();
                min_x = (min_x2 as f64 * scale_factor_x) as i32;
                max_x = (max_x2 as f64 * scale_factor_x) as i32;
                min_y = (min_y2 as f64 * scale_factor_y) as i32;
                max_y = (max_y2 as f64 * scale_factor_y) as i32;

                //println!("minx {} maxx {} miny {} maxy {}",min_x,max_x,min_y,max_y);
                let image = screen.capture_area(min_x as i32, min_y as i32, (max_x - min_x) as u32, (max_y - min_y) as u32).unwrap();
                let buffer = image.to_png(None).unwrap();
                data.image_data = buffer;

                let dynamic_image = load_from_memory_with_format(&data.image_data, image::ImageFormat::Png)
                    .expect("Failed to load image from memory");
                let rgba_image = dynamic_image.to_rgba8();
                let image_buf = ImageBuf::from_raw(
                    rgba_image.clone().into_raw(),
                    ImageFormat::RgbaSeparate,
                    rgba_image.clone().width() as usize,
                    rgba_image.clone().height() as usize,
                );

                let image = Image::new(image_buf);

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
                });

                let cancel_button = Button::new("Cancel").on_click(move |_ctx, _data: &mut GrabData ,_env| {
                    // cancel all image data
                    _data.image_data = vec![];
                });
                let rect = druid::Screen::get_monitors()[data.monitor_index].virtual_rect();
                //println!("{:?}",rect);
                //println!("{:?}",screen);
                ctx.window().close();
                ctx.new_window(
                    WindowDesc::new(
                        Flex::column().with_child(
                            ZStack::new(image)
                                .with_centered_child(ScreenshotWidget)
                        ).with_child(Flex::row().with_child(save_button).with_child(cancel_button))

                    )
                        .set_position((rect.x0,rect.y0))
                        .window_size(Size::new(rect.width()/2 as f64,rect.height()/2 as f64)));
            }
            //fs::write(format!("Screen{}.{}",data.screenshot_number,data.save_format), data.image_data.clone()).unwrap();
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