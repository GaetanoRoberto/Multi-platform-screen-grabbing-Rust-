use std::fs;
use std::fs::File;
use druid::{AppDelegate, commands, DelegateCtx, Env, Event, EventCtx, Widget, WindowDesc, WindowState};
use druid::widget::{Controller,Flex};
use serde_json::to_writer;
use crate::GrabData;
use crate::main_gui_building::build_ui;

pub struct Delegate;

impl AppDelegate<GrabData> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: druid::Target,
        cmd: &druid::Command,
        _data: &mut GrabData,
        _env: &druid::Env,
    ) -> druid::Handled {
        if cmd.is(commands::CLOSE_WINDOW) {
            // TODO: set initial value for parameters who need it
            // Handle the window close event
            println!("Closing the window");
            // cancel all image data
            _data.scale_factor = 1.0;
            _data.image_data = vec![];
            let file = File::create("settings.json").unwrap();
            to_writer(file, _data).unwrap();
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
            /*Event::KeyUp(key_event) => {
                println!("Key up: {:?}", key_event.key);
            }*/
            Event::KeyDown(key_event) => {
                if data.set_hot_key {
                    // capture and set the hotkey for screen grabbing
                    data.hotkey.push(key_event.key.to_string());
                    // println!("Key down: {:?}", key_event.key.to_string());
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
                        let screen = screenshots::Screen::all().unwrap()[data.monitor_index];
                        let image = screen.capture_area(0, 0, screen.display_info.width as u32, screen.display_info.height as u32).unwrap();
                        fs::write(format!("Screen{}.{}",data.screenshot_number,data.save_format), image.to_png(None).unwrap()).unwrap();
                        if data.screenshot_number == u32::MAX {
                            data.screenshot_number = 0;
                        } else {
                            data.screenshot_number+=1;
                        }
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