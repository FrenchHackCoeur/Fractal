extern crate num;

use num::Complex;
use std::str::FromStr;

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

fn main() {

}