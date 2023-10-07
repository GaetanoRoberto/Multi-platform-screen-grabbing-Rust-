use std::cell::RefCell;
use std::fs;
use std::fs::File;
use druid::{AppDelegate, BoxConstraints, commands, DelegateCtx, Env, Event, EventCtx, ImageBuf, InternalEvent, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Selector, TimerToken, UpdateCtx, Widget, WindowDesc, WindowState};
use druid::piet::ImageFormat;
use druid::widget::{Controller, Flex, Image};
use serde_json::to_writer;
use crate::{Annotation, GrabData};
use crate::main_gui_building::{build_ui, start_screening};
use crate::utilities::{load_image, reset_data, resize_image};

pub struct Delegate;

impl AppDelegate<GrabData> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: druid::Target,
        cmd: &druid::Command,
        data: &mut GrabData,
        _env: &druid::Env,
    ) -> druid::Handled {
        if cmd.is(commands::CLOSE_WINDOW) {
            // TODO: set initial value for parameters who need it
            // Handle the window close event
            println!("Closing the window");
            // reset data
            reset_data(data);
            let file = File::create("settings.json").unwrap();
            to_writer(file, data).unwrap();
            // the event keep processing and the window is closed
            return druid::Handled::No;
        }
        druid::Handled::No
    }
}

pub struct Enter;

impl<W: Widget<GrabData>> Controller<GrabData, W> for Enter {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &druid::Event, data: &mut GrabData, env: &Env) {

        match event {
            Event::WindowConnected => {
                ctx.request_focus();
            }
            Event::MouseDown(mouse) => {
                if mouse.button.is_left() {
                    data.input_timer_error.0 = false;
                    ctx.resign_focus();
                    ctx.request_focus();
                }
            }

            Event::KeyDown(key_event) => {
                if data.set_hot_key {
                    // capture and set the hotkey for screen grabbing
                    data.trigger_ui = !data.trigger_ui;
                    // avoid to add many times the same character if is being pressed
                    if data.hotkey_new.is_empty() {
                        data.hotkey_new.push(key_event.key.to_string());
                    } else if data.hotkey_new.len()>2{
                        data.input_hotkey_error.0 = true;
                        data.input_hotkey_error.1 = "Max 3 keys, confirm please".to_string();
                    } else if data.hotkey_new.len() >= 1 && !data.hotkey_new.contains(&key_event.key.to_string())  {
                        data.hotkey_new.push(key_event.key.to_string());
                        data.input_hotkey_error = (false,"".to_string());
                    }else{
                        data.input_hotkey_error.0 = true;
                        data.input_hotkey_error.1 = "Only distinct keys".to_string();
                    }
                } else {

                    // check current combination of the hotkey
                    if data.hotkey[data.hotkey_sequence] == key_event.key.to_string() {
                        data.hotkey_sequence+=1;
                    } else {
                        data.hotkey_sequence = 0;
                    }
                    // if the pressed keys corresponds to the hotkey combination, acquire the screen
                    if data.hotkey_sequence == data.hotkey.len() {
                        // acquire screen
                        start_screening(ctx,data.monitor_index,data);
                        data.hotkey_sequence = 0;
                    }
                }
            }
            _ => {} // Handle other cases if needed
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

pub struct NumericTextBoxController;

impl<W: Widget<GrabData>> Controller<GrabData, W> for NumericTextBoxController {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut GrabData, env: &druid::Env) {
        match event {
            Event::KeyDown(key_event) => {
                // remove error if widget lose focus (when pressing enter)
                if key_event.key.to_string() == "Enter" {
                    data.input_timer_error.0 = false;
                }
            }
            Event::KeyUp(key_event) => {
                // if lenght of the input field has not changed, it means that there is a wrong user input
                if !(data.delay_length == data.delay.len() && data.delay_length == 0 && key_event.key.to_string() == "Backspace") {
                    if data.delay_length == data.delay.len() {
                        data.input_timer_error.0 = true;
                        // set error message in case empty input happened
                        data.input_timer_error.1 = "Invalid Input: Only Positive Number are Allowed.".to_string();
                    } else {
                        // all ok, update the length of the field
                        data.input_timer_error.0 = false;
                        data.delay_length = data.delay.len();
                    }
                }
            }
            /*Event::Internal(internal_event) => {
                // Check if it's a timer event for a specific widget ID
                if let InternalEvent::RouteTimer(token, widget_id) = internal_event {
                    if *token == TimerToken::from_raw(data.timer_id) {
                        println!("Time elapsed: {} seconds",data.delay);
                        start_screening(ctx,data.monitor_index);
                    }
                } else {
                    // For other internal events, propagate them
                    child.event(ctx, event, data, env);
                }
            }*/
            _ => {
                // propagates other event in order to allow user input
                child.event(ctx, event, data, env);
            }
        }
    }
}

pub struct ImageSizeWidget {
    width: f64,
    height: f64,
}

impl ImageSizeWidget {
    // Constructor function to create an instance with default values
    pub fn new() -> Self {
        ImageSizeWidget {
            width: 0.0,  // Default width
            height: 0.0, // Default height
        }
    }
}

impl Widget<GrabData> for ImageSizeWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut GrabData, env: &Env) {
        match event {
            Event::Command(cmd) => {
                if cmd.is(druid::commands::SHOW_PREFERENCES) {
                    // Handle the custom command here
                    data.image_size.0 = self.width;
                    data.image_size.1 = self.height;
                    println!("{:?}", data.image_size);
                }
            }
            _ => {}
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &GrabData, env: &Env) {
        ctx.submit_command(druid::commands::SHOW_PREFERENCES);
        //println!("send command");
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &GrabData, data: &GrabData, env: &Env) {
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &GrabData, env: &Env) -> druid::Size {
        bc.max()
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, data: &GrabData, env: &Env) {
        let mut image = load_image(data);
        let rgba_image = image.to_rgba8();

        let image_buf = ImageBuf::from_raw(
            rgba_image.clone().into_raw(),
            ImageFormat::RgbaSeparate,
            rgba_image.clone().width() as usize,
            rgba_image.clone().height() as usize,
        );

        // Create an Image widget with your image
        let mut image = Image::new(image_buf);

        let image_width = rgba_image.width() as f64;
        let image_height = rgba_image.height() as f64;

        // Calculate the size of the image within the constraints of the Sized Box
        let constraints = paint_ctx.size(); // Get the constraints (size of the SizedBox)
        let mut scaled_width = image_width;
        let mut scaled_height = image_height;

        // Calculate the scaled size while preserving aspect ratio
        if image_width > constraints.width || image_height > constraints.height {
            let width_ratio = constraints.width / image_width;
            let height_ratio = constraints.height / image_height;
            let scale_factor = width_ratio.min(height_ratio);

            scaled_width *= scale_factor;
            scaled_height *= scale_factor;
        }

        image.paint(paint_ctx, data, env);

        // Store the actual rendered size of the image
        self.width = scaled_width;
        self.height = scaled_height;

        println!("box {}\nimage {:?}",constraints,(self.width,self.height));
    }
}