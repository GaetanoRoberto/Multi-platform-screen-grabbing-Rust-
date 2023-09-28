use std::fs;
use std::fs::File;
use druid::{AppDelegate, commands, DelegateCtx, Env, Event, EventCtx, InternalEvent, Selector, TimerToken, Widget, WindowDesc, WindowState};
use druid::widget::{Controller,Flex};
use serde_json::to_writer;
use crate::{Annotation, GrabData};
use crate::main_gui_building::{build_ui, start_screening};

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
            // cancel all image data
            data.first_screen = true;
            data.positions = vec![];
            data.scale_factor = 1.0;
            data.image_data_old = vec![];
            data.image_data_new = vec![];
            data.set_hot_key = false;
            data.annotation = Annotation::None;
            data.input_timer_error = (false,"".to_string());
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
                    if data.hotkey.is_empty() {
                        data.hotkey.push(key_event.key.to_string());
                    } else if data.hotkey.len() >= 1 && data.hotkey[data.hotkey.len()-1] != key_event.key.to_string() {
                        data.hotkey.push(key_event.key.to_string());
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
                        start_screening(ctx,data.monitor_index);
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