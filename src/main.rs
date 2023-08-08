use std::fs;
use druid::{Data, Widget, LocalizedString, WindowDesc, AppLauncher, PlatformError, widget::{Label, Button, Flex}, WidgetExt};
use screenshots::Screen;
use druid::widget::List;

#[derive(Clone, Data)]
struct GrabData {
    monitor_index: u32,
    save_path: String
}

fn build_ui() -> impl Widget<GrabData> {
    let title = Label::new("Screen Grabbing Utility");

    let screenshot_button = Button::new("Take a Screenshot").on_click(
        |_ctx, data: &mut GrabData ,_env| {
            let screens = Screen::all().unwrap();
            let screen = screens[data.monitor_index as usize];
            let image = screen.capture().unwrap();
            let buffer = image.to_png(None).unwrap();
            fs::write(format!("{}.jpeg", screen.display_info.id), buffer).unwrap();
        }
    );

    Flex::column().with_child(title).with_child(screenshot_button)
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(build_ui())
        .title("Screen grabbing Utility")
        .window_size((400.0, 300.0));
    AppLauncher::with_window(main_window).launch(GrabData { monitor_index: 0, save_path: "C:\\Users\\Domenico\\Desktop".to_string() })
}
