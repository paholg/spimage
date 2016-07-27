use std::f32::consts::PI;
use spin::{NLEDS, R0, LED_SIZE, LED_SPACE};

use image::open;

use std::path::Path;
use std::error::Error;
pub fn convert_image(path: &Path, nslices: usize) -> Result<String, Box<Error>> {
    let img = try!(open(path)).to_rgb();

    let (width, height) = img.dimensions();
    let (cx, cy) = (width/2, height/2);

    let radius_pixels = ::std::cmp::min(width, height) as f32 / 2.0;
    let radius_mm = R0 + LED_SIZE*NLEDS as f32 + LED_SPACE*(NLEDS - 1) as f32;

    let r0 = R0 / radius_mm * radius_pixels;
    let dr = (LED_SIZE + LED_SPACE) / 2.0 / radius_mm * radius_pixels;

    let dphi = 2.0 * PI / nslices as f32;
    let phi0 = 0.0;


    use spin::color::Rgb;
    let mut polar_img: Vec<(f32, [Rgb; NLEDS])> = Vec::with_capacity(nslices);
    // Here, we want to construct a pineapple chunk that has corners (r - dr/2, phi - dphi/2) and
    // (r + dr/2, phi + dphi/2). We then want to find all pixels it overlaps with, and how much
    // they overlap (where 1.0 is a full overlap). Finally, we take a weighted average of them, and
    // that is our value for coordinate (r, phi).

    // doing hokey bad stuff for now
    for phi_i in 0..nslices {
        let phi = phi0 + dphi * phi_i as f32;

        let mut slice = [Rgb::default(); NLEDS];
        for r_i in 0..NLEDS {
            let r = r0 + dr * r_i as f32;
            println!("r: {}, phi: {}", r, phi);
            let x = ((r * phi.cos()).round() as i32 + cx as i32) as u32;
            let y = ((r * phi.sin()).round() as i32 + cy as i32) as u32;
            let pixel = img.get_pixel(x, y);

            slice[r_i] = Rgb::new(pixel[0], pixel[1], pixel[2]);
        }
        polar_img.push((phi, slice));
    }

    let mut result = String::new();
    use std::fmt::Write;
    try!(result.write_str("&["));

    for (phi, slice) in polar_img {
        try!(write!(result, "({:.8}, [", phi));
        for color in &slice {
            try!(write!(result, "Rgb::new({}, {}, {}), ", color.r, color.g, color.b));
        }
        try!(write!(result, "]),\n"));
    }
    try!(write!(result, "];"));

    Result::Ok(result)
}


// fn main() {
//     let text = convert_image(Path::new("imgs/test_img.png"), 72).unwrap();
//     let dest = Path::new("examples/picture.dat");
//     use std::fs::File;

//     let mut f = File::create(dest).unwrap();
//     use std::io::Write;
//     f.write_all(text.as_bytes()).unwrap();
// }
