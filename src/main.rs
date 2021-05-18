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
    let mut p = &v[0];
    v[1..]
        .iter()
        .map(|c| {
            let d = c - p;
            p = c;
            d
        })
        .collect::<Vec<f64>>()
}

/*
fn smooth_differentiate(v: &[f64], l: usize) -> Vec<f64> {
    (l..v.len()-l).map(|i| -> f64 {
        (v[i..i+l].iter().sum::<f64>() / l as f64) - (v[i-l..i].iter().sum::<f64>() / l as f64)
    }).collect::<Vec<f64>>()
}
*/

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

    let impulse_len = 8 * sigma;
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
        } else {
            let mut q = convolve(&v, &impulse, 1.0);
            for _i in 0..nth_derivative {
                q = differentiate(&q);
            }
            q
        };
        //let res = &_res[impulse_len-1.._res.len()-impulse_len];
        let adj = if raw_signal {
            0
        } else {
            ((impulse_len / 2) * (nth_derivative + 1)) as i64
        };
        res.iter()
            .enumerate()
            .for_each(|(i, x)| println!("{}\t{}\t{}\t{}", name, sigma, (i as i64 - adj), x));
    });
}
