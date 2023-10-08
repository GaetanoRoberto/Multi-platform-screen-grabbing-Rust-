// IMAGE SCREEN FUNCTIONS

use std::f64::consts::PI;
use druid::{EventCtx, MouseEvent, Point, Vec2};
use image::{DynamicImage, load_from_memory_with_format};
use screenshots::Screen;
use crate::{Annotation, GrabData};
use crate::constants::{BUTTON_HEIGHT, NORMAL_BIG_IMAGE_LIMIT, SMALL_IMAGE_LIMIT};
use image::imageops::FilterType;

pub fn rescale_coordinates(ctx: &mut EventCtx, mouse_event: &MouseEvent, data: &mut GrabData) {
    let mut image = load_image(data);
    // Calculate the offset to center mouse positions in the Image
    let widget_size = ctx.size();
    println!("rescale window {}",widget_size);
    let image_width = data.image_size.0;
    let image_height = data.image_size.1;
    let x_offset = (widget_size.width - image_width) / 2.0;
    let y_offset = (widget_size.height - image_height) / 2.0;
    let width_scale = image_width / widget_size.width;
    let height_scale = image_height / widget_size.height;
    //println!("parent {} {}",widget_size.width, widget_size.height);
    println!("offswt {} {}",x_offset,y_offset);
    // save corresponding offset to subtract in rectangle paint function
    data.offsets.push(<(f64, f64)>::from((x_offset,y_offset)));
    // Adjust mouse coordinates
    let mut centered_pos = mouse_event.pos - Vec2::new(x_offset, y_offset);
    centered_pos.x = centered_pos.x * width_scale;
    centered_pos.y = centered_pos.y * height_scale;
    //println!("Coordinates: {}",mouse_event.pos);
    //println!("centered coordinates: {}",centered_pos);
    data.positions.push(<(f64, f64)>::from(centered_pos));
}

pub fn load_image(data: &GrabData ) -> DynamicImage {
    let mut image;

    if data.image_data_new.is_empty() {
        image = load_from_memory_with_format(&data.image_data_old, image::ImageFormat::Png)
            .expect("Failed to load image from memory");
    } else {
        image = load_from_memory_with_format(&data.image_data_new, image::ImageFormat::Png)
            .expect("Failed to load image from memory");
    }

    image
}

pub fn image_to_buffer(image: DynamicImage) -> Vec<u8> {
    let mut png_buffer = std::io::Cursor::new(Vec::new());
    image.write_to(&mut png_buffer, image::ImageFormat::Png)
        .expect("Failed to Save Modified Image");

    png_buffer.into_inner()
}

pub fn make_rectangle_from_points(data: &GrabData ) -> Option<(f64,f64,f64,f64)> {
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

pub fn compute_circle_center_radius(min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> (f64,f64,f64) {
    // compute the center
    let center_x = (max_x - min_x) as f64 / 2.0 + min_x as f64;
    let center_y = (max_y - min_y) as f64 / 2.0 + min_y as f64;
    let radius = (((max_x - min_x).pow(2) + (max_y - min_y).pow(2)) as f64).sqrt()/ 2.0;

    (center_x,center_y,radius)
}

pub fn compute_arrow_points(data: &GrabData) -> Option<((Point,Point),(Point,Point),(Point,Point))> {
    if data.positions.is_empty() {
        return None;
    }
    let main_line_p0 = Point::new(data.positions[0].0, data.positions[0].1);
    let main_line_p1 = Point::new(data.positions[data.positions.len()-1].0, data.positions[data.positions.len()-1].1);

    //direzione = endX - startX , endY - startY
    let direction = Point::new(data.positions[data.positions.len() - 1].0 - data.positions[0].0, data.positions[data.positions.len() - 1].1 - data.positions[0].1);
    //lunghezza = ipotenusa teorema di pitagora
    let arrow_length = ((direction.x.powi(2) + direction.y.powi(2)) as f64).sqrt();
    // angolo tra asseX e freccia
    let angle = (direction.y as f64).atan2(direction.x as f64);
    // lunghezza punta della freccia [settata ad un terzo]
    let arrow_tip = arrow_length/3.0;

    // Calcola punti della punta della freccia
    let arrow_x1 = data.positions[data.positions.len() - 1].0 - (direction.x / arrow_length);
    let arrow_y1 = data.positions[data.positions.len() - 1].1 - (direction.y / arrow_length);

    let arrow_l0_p0 = main_line_p1;
    let arrow_l0_p1 = Point::new(arrow_x1 - arrow_tip * (angle + PI / 6.0).cos(),arrow_y1 - arrow_tip * (angle + PI / 6.0).sin());
    let arrow_l1_p0 = main_line_p1;
    let arrow_l1_p1 = Point::new(arrow_x1 - arrow_tip * (angle - PI / 6.0).cos(),arrow_y1 - arrow_tip * (angle - PI / 6.0).sin());

    // main line point couple, first line point couple, second line point couple
    Some(((main_line_p0,main_line_p1),(arrow_l0_p0, arrow_l0_p1),(arrow_l1_p0, arrow_l1_p1)))
}

pub fn compute_highlighter_points(data: &GrabData) -> Option<(Point, Point, Point, Point)> {
    if data.positions.is_empty() {
        return None;
    }
    // draw line with first and last position, then clear the vector
    let point1 = Point::new(data.positions[0].0, data.positions[0].1);
    let point2 = Point::new(data.positions[data.positions.len()-1].0,data.positions[data.positions.len()-1].1);

    // Define your margin and the two points representing the line segment
    let highlighter_width = data.highlighter_width;

    // Calculate the slope of the line
    let dx = point2.x - point1.x;
    let dy = point2.y - point1.y;
    let slope = dy / dx;

    // Calculate the angle of the line with respect to the horizontal axis
    let angle = slope.atan();

    // Calculate the change in x and y coordinates for the margin
    let delta_x = highlighter_width * (angle + PI / 2.0).cos();
    let delta_y = highlighter_width * (angle + PI / 2.0).sin();

    // Create the four vertices of the rectangle
    let rect_point1 = Point::new(point1.x + delta_x, point1.y + delta_y);
    let rect_point2 = Point::new(point1.x - delta_x, point1.y - delta_y);
    let rect_point3 = Point::new(point2.x - delta_x, point2.y - delta_y);
    let rect_point4 = Point::new(point2.x + delta_x, point2.y + delta_y);

    Some((rect_point1,rect_point2,rect_point3,rect_point4))
}

// Image Resizing
pub fn resize_image(mut image: DynamicImage, data: &mut GrabData) -> (f64, f64) {
    let screen = Screen::all().unwrap()[data.monitor_index];
    let mut scale_factor_x ;
    let mut scale_factor_y;

    if image.width() >= (screen.display_info.width as f64 * NORMAL_BIG_IMAGE_LIMIT) as u32 || image.height() >= (screen.display_info.height as f64 * NORMAL_BIG_IMAGE_LIMIT) as u32 {
        // NORMAL OR BIG IMAGE (>= 50% of the screen)
        scale_factor_x = image.width() as f64 / (screen.display_info.width as f64 * 1.4);
        scale_factor_y = image.height() as f64 / (screen.display_info.height as f64 * 1.4);

    } else if image.width() <= (screen.display_info.width as f64 * SMALL_IMAGE_LIMIT) as u32 && image.height() <= (screen.display_info.height as f64 * SMALL_IMAGE_LIMIT) as u32 {
        // VERY SMALL IMAGE (<= 20% of the screen)
        println!("small small");
        scale_factor_x = 0.25;
        scale_factor_y = 0.25;
        image = image.resize((screen.display_info.width / 4), (screen.display_info.height / 4), FilterType::Nearest);
    }else{
        // SMALL IMAGE (20% of the screen < size < 50% of the screen)
        println!("small");
        scale_factor_x = (image.width() as f64 * 1.4) / (screen.display_info.width as f64);
        scale_factor_y = (image.height() as f64 * 1.4) / (screen.display_info.height as f64);
    }

    let aspect_ratio = image.width() as f64 / image.height() as f64;
    let desired_width = (screen.display_info.width as f64) * scale_factor_x;
    let desired_height = (screen.display_info.height as f64 - 7.0 * BUTTON_HEIGHT) * scale_factor_y;
    // Calculate the scaled dimensions while preserving aspect ratio
    let (scaled_width, scaled_height) = if image.width() as f64 / desired_width > image.height() as f64 / desired_height {
        // Fit by width
        (desired_width, desired_width / aspect_ratio as f64)
    } else {
        // Fit by height
        (desired_height * aspect_ratio as f64, desired_height)
    };

    data.image_size = (scaled_width,scaled_height);
    // assign scale factors to data
    data.scale_factors.push(((image.width() as f64 / scaled_width),(image.height() as f64 / scaled_height)));
    //data.scale_factors.0 = image.width() as f64 / scaled_width;
    //data.scale_factors.1 = image.height() as f64 / scaled_height;
    println!("{:?}",data.scale_factors);


    (scaled_width,scaled_height)
}

// Reset Data Function
pub fn reset_data(data: &mut GrabData) {
    // set data fields to their initial state
    data.image_data_old = vec![];
    data.image_data_new = vec![];
    data.press = false;
    data.first_screen = true;
    data.scale_factors = vec![];
    data.positions = vec![];
    data.offsets = vec![];
    data.hotkey_new = vec![];
    data.hotkey_sequence = 0;
    data.set_hot_key = false;
    data.input_timer_error = (false,"Invalid Input: Only Positive Number are Allowed.".to_string());
    data.input_hotkey_error = (false,"Invalid Input: Wrong Hotkey.".to_string());
    data.trigger_ui = false;
    data.annotation = Annotation::None;
    data.text_annotation = "".to_string();
}