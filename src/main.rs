use flate2::bufread::GzDecoder;
use std::fs::File;
use std::io;
use std::io::{prelude::*, BufReader};
use compute::signal::*;
use compute::distributions::Normal;
use std::f64::consts::{E , PI};
// // use num::pow::pow;
// // use crate::num::traits::Pow;
// extern crate num;
//
// use num_traits::pow;


extern crate clap;
use clap::{App, Arg};

fn for_each_line_in_file(filename: &str, mut callback: impl FnMut(&str)) {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        callback(&line.unwrap());
    }
}

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

fn normal_pdf(mu : f64 , sigma : f64, x : f64)-> f64{
    // Can also explicitly define type i.e. i32
    E.powf(-1.0/2.0 * (((x-mu)/sigma).powf(2.0)) / (sigma * (2.0 * PI).sqrt() ))
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
                .help("sigma for our gaussian"),
        )
        .get_matches();

    let sigma = matches
        .value_of("sigma")
        .unwrap_or(&"1")
        .parse::<usize>()
        .unwrap();    // let dist  = Normal::new(0.0, 100.0);
    let impulse_len = 3*sigma;
    let mu = impulse_len/2;
    let impulse = (0..impulse_len).map(|x| normal_pdf(mu as f64, sigma as f64, x as f64)).collect::<Vec<f64>>();
    // println!("{}", impulse.iter().map(|x| format!("{}", x)).collect::<Vec<String>>().join(" "));

    let filename = matches.value_of("INPUT").unwrap();
    for_each_input_vector(filename, |name, v| {
        // println!("{} {}", name, v.iter().sum::<f32>() / (v.len() as f32))

        let res = convolve(&v, &impulse, 0.5);
        println!("{}", res.iter().map(|x| format!("{}", x)).collect::<Vec<String>>().join(" "))


    });
}
