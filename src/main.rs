extern crate num;
extern crate image;
extern crate crossbeam;
extern crate num_cpus;

use num::Complex;
use std::str::FromStr;
use std::fs::File;
use std::io::Write;
use image::ColorType;
use image::png::PNGEncoder;
use num::traits::real::Real;
use std::thread::spawn;


/// This function aims to determine whether or not the 'c' parameter belongs to the Mandelbrot
/// set using a limited number of rounds to decide.
///
/// If 'c' is not an element of the Mandelbrot set the function will return Some(i) where i
/// corresponds to the round from which the norm of complex number z is greater than or equal to 2
/// otherwise it will return None.
fn escape_time(c: Complex<f64>, limit: u32) -> Option<u32> {
    let mut z = Complex {re: 0.0, im: 0.0};

    for i in 0..limit {
        z = z * z + c;

        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
    }

    None
}


/// This function aims to parse the string s which must match the following pattern
/// <left><sep><right> where <sep> corresponds to the 'separator' parameter and where both <left>
/// and <right> correspond to a string which can be processed by 'T::from_str'.
fn pair_analyze<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index+1..])) {
                (Ok(l), Ok(r)) => Some((l, r)),
                _ => None
            }
        }
    }
}

#[test]
fn pair_analyze_test () {
    assert_eq!(pair_analyze::<i32>("", ','), None);
    assert_eq!(pair_analyze::<i32>("10,", ','), None);
    assert_eq!(pair_analyze::<i32>(",10", ','), None);
    assert_eq!(pair_analyze::<i32>("10,20", ','), Some((10, 20)));
    assert_eq!(pair_analyze::<i32>("10,20xy", ','), None);
    assert_eq!(pair_analyze::<f64>("0.5x", 'x'), None);
    assert_eq!(pair_analyze::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
}


/// This function aims to analyse a pair of floating number separated by a comma that
/// should represent a complex number.
fn complex_pair_analyze(s: &str) -> Option<Complex<f64>> {
    match pair_analyze(s, ',') {
        Some((re, im)) => Some(Complex {re, im}),
        None => None
    }
}

#[test]
fn complex_pair_analyze_test() {
    assert_eq!(complex_pair_analyze(",-0.0625"), None);
    assert_eq!(complex_pair_analyze("1.25,-0.0625"), Some(Complex {re: 1.25, im: -0.0625}));
}


/// Starting from the row and the column of a pixel of the output image, the function will
/// return the correspond point in the complex plane.
///
/// 'edges' is a tuple that contains the width and the height of the output image in pixels.
/// 'super_left' and 'infer_right' are both points in the complex plane bounding the output image
/// areas.
fn pixel_to_complex_point(
    edges: (usize, usize),
    pixel: (usize, usize),
    super_left: Complex<f64>,
    infer_right: Complex<f64>) -> Complex<f64> {

    let (width, height) = (infer_right.re - super_left.re, super_left.im - infer_right.im);

    Complex {
        re: super_left.re + ( pixel.0 as f64 / edges.0 as f64) * width,
        im: super_left.im - ( pixel.1 as f64 / edges.1 as f64) * height
    }
}

#[test]
fn pixel_to_complex_point_test() {
    assert_eq!(pixel_to_complex_point(
        (100, 100),
        (25, 75),
        Complex { re: -1.0, im: 1.0},
        Complex { re: 1.0, im: -1.0}
    ), Complex { re: -0.5, im: -0.5})
}

/// Filling an array of pixels to represent a Mandelbrot rectangle.
///
/// 'edges' is a parameter to indicate the width and the height of the output image.
/// 'super_left' and 'infer_right' correspond respectively to the top left corner and the bottom
/// right corner of the output image.
fn render(
    pixels: &mut [u8],
    edges: (usize, usize),
    super_left: Complex<f64>,
    infer_right: Complex<f64>
) {
    assert_eq!(pixels.len(), edges.0 * edges.1);

    for row in 0..edges.1 {
        for column in 0..edges.0 {
            let point = pixel_to_complex_point(
                edges,
                (column, row),
                super_left,
                infer_right
            );

            pixels[row * edges.0 + column] = match escape_time(point, 255) {
                None => 0,
                Some(count) => 255 - count as u8,
            }
        }
    }
}


/// This function stores a Mandelbrot rectangle contained in 'pixels' of resolution 'edges' in a
/// file named 'file_name'.
fn save_mandelbrot_rectangle_as_png(
    file_name: &str,
    pixels: &[u8],
    edges: (usize, usize)
) -> Result<(), std::io::Error> {
    let output_file = File::create(file_name)?;
    let encoder = PNGEncoder::new(output_file);
    encoder.encode(
        &pixels,
        edges.0 as u32,
        edges.1 as u32,
        ColorType::Gray(8)
    )?;

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        writeln!(std::io::stderr(), "Usage: mandelbrot FILE_NAME PIXELS SUP_LEFT INFER_RIGHT").unwrap();
        writeln!(std::io::stderr(), "Example: {} mandelbrot.png 1000x750 -1.20,0.60 -1,0.20", args[0]).unwrap();
        std::process::exit(1);
    }

    let edges = pair_analyze(&args[2], 'x')
        .expect("Incorrect value for the resolution of the output image");

    let super_left = complex_pair_analyze(&args[3])
        .expect("Incorrect format for the top left corner complex point");
    let infer_right = complex_pair_analyze(&args[4])
        .expect("Incorrect format for the bottom right corner complex point");

    let mut pixels = vec![0; edges.0 * edges.1];

    let cpus = num_cpus::get();
    let rows_per_chunk = edges.1 / cpus + 1;

    {
        let chunks: Vec<&mut [u8]> = pixels.chunks_exact_mut(rows_per_chunk * edges.0)
            .collect();

        crossbeam::scope(|spawner| {
           for (i, chunk) in chunks.into_iter().enumerate() {
               let top = rows_per_chunk * i;
               let height = chunk.len() / edges.0;
               let chunk_shape = (edges.0, height);
               let chunk_supl = pixel_to_complex_point(
                   edges,
                   (0, top),
                   super_left,
                   infer_right
               );
               let chunk_infr = pixel_to_complex_point(
                   edges,
                   (edges.0, top + height),
                   super_left,
                   infer_right
               );

               spawner.spawn(move || {
                   render(chunk, chunk_shape, chunk_supl, chunk_infr);
               });
           }
        });
    }

    save_mandelbrot_rectangle_as_png(&args[1], &pixels, edges).expect("An error \
        occured while trying to save the mandelbrot rectangle");
}