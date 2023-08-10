use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use druid::{ImageBuf, Application, Color, Data, Widget, LocalizedString, WindowDesc, AppLauncher, PlatformError, widget::{Image, Label, Button, Flex}, WidgetExt, AppDelegate, DelegateCtx, WindowId, piet};
use druid::piet::ImageFormat;
use screenshots::Screen;
use serde::{Serialize,Deserialize};
use serde_json::{to_writer,from_reader};
use image::{open, DynamicImage, ImageBuffer, Rgba, GenericImageView, load_from_memory_with_format};

#[derive(Clone, Data, Serialize, Deserialize)]
struct GrabData {
    screenshot_number : u32,
    #[data(ignore)]
    image_data: Vec<u8>,
    save_path: String,
    save_format : String
}

fn create_monitor_buttons() -> Flex<GrabData> {
    let screens = Screen::all().unwrap();
    let mut monitor_buttons = Flex::column();
    let mut monitor_index = 1;
    for screen in screens {
        let btn = Button::new( "📷 Monitor ".to_owned() + &monitor_index.to_string()).on_click(
            move |_ctx, _data: &mut GrabData ,_env| {
                let image = screen.capture().unwrap();
                let buffer = image.to_png(None).unwrap();
                _data.image_data = buffer;
                /* SAVE LOGIC
                fs::write(format!("Screen{}.{}",_data.screenshot_number,_data.save_format), buffer).unwrap();
                if _data.screenshot_number == u32::MAX {
                    _data.screenshot_number = 0;
                } else {
                    _data.screenshot_number+=1;
                }
                SAVE LOGIC */
            }
        );
        monitor_buttons = monitor_buttons.with_child(btn);
        monitor_index+=1
    }
    monitor_buttons
}

fn build_ui() -> impl Widget<GrabData> {
    //let title = Label::new("Screen Grabbing Utility");
    /*let dynamic_image = open("C:\\Users\\Domenico\\CLionProjects\\pds_project\\Screen1.png").unwrap();
    // Create a DynamicImage from the image data buffer
    //let dynamic_image = load_from_memory_with_format(&image_data, ImageFormat::Png)
    //    .expect("Failed to load image from memory");
    let rgba_image = dynamic_image.to_rgba8();
    let image_buf = ImageBuf::from_raw(
        rgba_image.clone().into_raw(),
        ImageFormat::RgbaSeparate,
        rgba_image.clone().width() as usize,
        rgba_image.clone().height() as usize,
    );
    let image = Image::new(image_buf);*/

    Flex::column().with_child(create_monitor_buttons())
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(build_ui())
        .title("Screen grabbing Utility")
        .window_size((400.0, 300.0));

    let file = File::open("settings.json").unwrap();
    let data: GrabData = from_reader(file).unwrap();
    /*let data = GrabData {
        screenshot_number: 1,
        image_data: vec![],
        save_path: "C:\\Users\\Domenico\\Desktop".to_string(),
        save_format: "png".to_string()
    };*/

    AppLauncher::with_window(main_window).delegate(Delegate).launch(data)
}

struct Delegate;

impl AppDelegate<GrabData> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: druid::Target,
        cmd: &druid::Command,
        _data: &mut GrabData,
        _env: &druid::Env,
    ) -> druid::Handled {
        if cmd.is(druid::commands::CLOSE_WINDOW) {
            // Handle the window close event
            println!("Closing the window");
            _data.image_data = vec![];
            let file = File::create("settings.json").unwrap();
            to_writer(file, _data).unwrap();
            // the event keep processing and the window is closed
            return druid::Handled::No;
        }
        druid::Handled::No
    }
}