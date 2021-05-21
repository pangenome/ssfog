use compute::signal::*;
use flate2::bufread::GzDecoder;
use std::fs::File;
use std::io::{prelude::*, BufReader};
//use compute::distributions::Normal;
use std::f64::consts::{E, PI};
// // use num::pow::pow;
// // use crate::num::traits::Pow;
// extern crate num;
//
// use num_traits::pow;

extern crate clap;
use clap::{App, Arg};

/*
fn for_each_line_in_file(filename: &str, mut callback: impl FnMut(&str)) {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        callback(&line.unwrap());
    }
}
*/

fn for_each_line_in_gzfile(filename: &str, mut callback: impl FnMut(&str)) {
    let file = File::open(filename).unwrap();
    let gz = GzDecoder::new(BufReader::new(file));
    let reader = BufReader::new(gz);
    for line in reader.lines() {
        callback(&line.unwrap());
    }
}

fn for_each_input_vector(filename: &str, mut callback: impl FnMut(&str, Vec<f64>)) {
    for_each_line_in_gzfile(filename, |line| {
        let mut it = line.split(' ');
        let name = it.next().unwrap();
        let v = it.map(|s| s.parse::<f64>().unwrap()).collect();
        callback(name, v);
    })
}

fn normal_pmf(mu: f64, sigma: f64, x: f64) -> f64 {
    // Can also explicitly define type i.e. i32
    E.powf(-((x - mu) / sigma).powf(2.0) / 2.0) / (sigma * (2.0 * PI).sqrt())
}

fn differentiate(v: &[f64]) -> Vec<f64> {
    v[..v.len() - 1]
        .iter()
        .enumerate()
        .map(|(i, c)| v[i + 1] - c)
        .collect::<Vec<f64>>()
}

fn zero_crossings(v: &[f64]) -> Vec<(usize, f64)> {
    let mut p = &v[0];
    let zero = &0f64;
    v[1..]
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let d = if p < zero && c > zero || p > zero && c < zero {
                Some((i, c - p))
            } else {
                None
            };
            p = c;
            d
        })
        .collect::<Vec<(usize, f64)>>()
}

fn multiple_zero_crossing(v: &Vec<f64>, s: &str, n: usize)  {
    let sigma_string: Vec<&str> = s.split(',').collect();
    let sigma_range_start= sigma_string[0].parse::<i32>().unwrap();
    let sigma_range_end= sigma_string[2].parse::<i32>().unwrap();
    let sigma_range_step= sigma_string[1].parse::<usize>().unwrap();
    for temp_sigma in (sigma_range_start..sigma_range_end).step_by(sigma_range_step) {

        let impulse_len = 6 * temp_sigma;
        let mu = impulse_len / 2;
        let raw_impulse = (0..impulse_len)
            .map(|x| normal_pmf(mu as f64, temp_sigma as f64, x as f64))
            .collect::<Vec<f64>>();
        let impulse_weight: f64 = raw_impulse.iter().sum();
        let impulse = raw_impulse
            .iter()
            .map(|x| x / impulse_weight)
            .collect::<Vec<f64>>();
        let res = {
            let mut q = convolve(&v, &impulse, 1.0);
            for _i in 0..n  {
                q = convolve(&differentiate(&q), &impulse, 1.0);
            }
            q
        };
        let adj = ((impulse_len / 2) * (n as i32 + 1)) as i64;
        zero_crossings(&res).iter().for_each(|(i, m)| {
            println!("{}\t{}\t{}", (*i as i64 - adj), m, &temp_sigma);
            });
        }
    }


fn main() {
    let matches = App::new("ssfog")
        .version("0.1.0")
        //.author("Erik Garrison <erik.garrison@gmail.com>")
        .about("scale-space filtering on graph coverage vectors")
        .arg(
            Arg::with_name("INPUT")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("input coverage vector file"),
        )
        .arg(
            Arg::with_name("sigma")
                .short("s")
                .long("sigma")
                .takes_value(true)
                .help("sigma for our gaussian"),
        )
        .arg(
            Arg::with_name("nth-derivative")
                .short("d")
                .long("nth-derivative")
                .takes_value(true)
                .help("return the nth derivative of the signal"),
        )
        .arg(
            Arg::with_name("raw-signal")
                .short("r")
                .long("raw-signal")
                .help("write the input signal without processing"),
        )
        .arg(
            Arg::with_name("zero-cross")
                .short("z")
                .long("zero-cross")
                .help("Combined to -d 2 retrive the zero-crossing position for specific sigma"),
        )
        .arg(
            Arg::with_name("sigma-range")
                .short("sr")
                .long("sigma-range")
                .takes_value(true)
                .help("range for reiterate the software with different sigma. Format : 10,5,100. From 10 to 100 by 5 step."),
        )
        .arg(
            Arg::with_name("multiple-zero-cross")
                .short("mz")
                .long("multiple-zero-cross")
                .help("Create multiple zero_crossing vector for a range of Sigma"),
        )
        .get_matches();

    let sigma = matches
        .value_of("sigma")
        .unwrap_or(&"1")
        .parse::<usize>()
        .unwrap();
    let nth_derivative = matches
        .value_of("nth-derivative")
        .unwrap_or(&"0")
        .parse::<usize>()
        .unwrap();
    let raw_signal = matches.is_present("raw-signal");
    let zero_cross = matches.is_present("zero-cross");
    let sigma_range = matches
        .value_of("sigma-range")
        .unwrap();
    let multiple_zero_cross = matches.is_present("multiple-zero-cross");

    let impulse_len = 6 * sigma;
    let mu = impulse_len / 2;
    let raw_impulse = (0..impulse_len)
        .map(|x| normal_pmf(mu as f64, sigma as f64, x as f64))
        .collect::<Vec<f64>>();
    let impulse_weight: f64 = raw_impulse.iter().sum();
    let impulse = raw_impulse
        .iter()
        .map(|x| x / impulse_weight)
        .collect::<Vec<f64>>();
    //impulse.iter().enumerate().for_each(|(i, x)| { println!("{} {}", i, x);});
    // println!("{}", impulse.iter().map(|x| format!("{}", x)).collect::<Vec<String>>().join(" "));

    let filename = matches.value_of("INPUT").unwrap();
    for_each_input_vector(filename, |name, v| {
        let res = if raw_signal {
            v
        } else if nth_derivative == 0 {
            convolve(&v, &impulse, 1.0)
        } else if multiple_zero_cross {
            v
        } else {
            let mut q = convolve(&v, &impulse, 1.0);
            for _i in 0..nth_derivative {
                q = convolve(&differentiate(&q), &impulse, 1.0);
            }
            q
        };

        if multiple_zero_cross {
            multiple_zero_crossing(&res, &sigma_range, nth_derivative)
        }

        let adj = if raw_signal {
            0
        } else {
            ((impulse_len / 2) * (nth_derivative + 1)) as i64
        };
        if zero_cross {
            zero_crossings(&res).iter().for_each(|(i, m)| {
                println!("{}\t{}\t{}", (*i as i64 - adj), m, &sigma);
            });
        }  else {
            //let res = &_res[impulse_len-1.._res.len()-impulse_len];
            res.iter()
                .enumerate()
                .for_each(|(i, x)| println!("{}\t{}\t{}\t{}", name, sigma, (i as i64 - adj), x));
        };
    });
}
