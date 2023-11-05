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
            // Handle the window close event
            println!("Closing the window");
            // create a data copy to save into json, without actually modifying data (NEEDED FOR LINUX)
            let json_data = GrabData {
                screenshot_number: data.screenshot_number,
                monitor_index: data.monitor_index,
                image_data_old: vec![],
                image_data_new: vec![],
                save_path: data.save_path.clone(),
                save_format: data.save_format.clone(),
                press: false,
                first_screen: true,
                scale_factors: (1.0, 1.0),
                image_size: data.image_size,
                positions: vec![],
                offsets: (0.0, 0.0),
                hotkey: data.hotkey.clone(),
                hotkey_new: vec![],
                hotkey_pressed: vec![],
                set_hot_key: false,
                delay: data.delay.clone(),
                input_hotkey_error: (false,"Invalid Input: Wrong Hotkey.".to_string()),
                trigger_ui: false,
                annotation: Annotation::None,
                color: data.color,
                text_annotation: "".to_string(),
                text_size: data.text_size,
                highlighter_width: data.highlighter_width,
            };
            let file = File::create("settings.json").unwrap();
            to_writer(file, &json_data).unwrap();
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
            Event::KeyUp(event) => {
                if data.hotkey.contains(&event.key.to_string()) {
                    data.hotkey_pressed = vec![];
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

                    // check key of hotkey not yet pressed
                    if data.hotkey.contains(&key_event.key.to_string())   {

                        if !data.hotkey_pressed.contains(&key_event.key.to_string()) {
                            data.hotkey_pressed.push(key_event.key.to_string());
                        }
                    }else {
                        data.hotkey_pressed = vec![];
                    }
                    // if the pressed keys corresponds to the hotkey combination, acquire the screen
                    if data.hotkey_pressed.len() == data.hotkey.len() {
                        // acquire screen
                        start_screening(ctx,data.monitor_index,data);
                        data.hotkey_pressed = vec![];
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