#![allow(unused_imports)]

extern crate anyhow;
extern crate cpal;
extern crate hound;

use std::fs::File;
use std::io;
use rodio::source::Source;
use rodio::buffer::SamplesBuffer;
use device_query::{DeviceQuery, DeviceState, Keycode};

mod draw_graph;
use draw_graph::draw_spectrum;

mod record;
use record::record;

mod world;
use world::{
    change_pitch,
    change_speed,
    change_spectral_envelope,
    to_robot,
    to_female,
    to_mosaic,
    synthesis,
    wav2world,
};

mod cui;
use cui::{
    main_menu,
    pitch_menu,
    speed_menu,
    spectral_menu,
    volume_menu,
};

fn main() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/recorded.wav");
    let device = rodio::default_output_device().unwrap();
    let sink = rodio::Sink::new(&device);
    let device_state = DeviceState::new();

    println!("\nStart with [Space] key...");
    loop {
	let mut keys = device_state.get_keys();
	if !keys.contains(&Keycode::Space) {
	    continue;
	}
	record(path, 3).unwrap();
	let file = File::open(path).unwrap();
	let source  = rodio::Decoder::new(io::BufReader::new(file)).unwrap().buffered();
	let fs = source.sample_rate();
	
	let even = source.clone().map(|d| d as f64).skip(1).step_by(2);
	let data: Vec<f64> = source.clone().map(|d| d as f64).step_by(2).zip(even).map(|(o, e)| e*0.5+o*0.5).collect();
	#[allow(unused_mut)]
	let (mut f0, mut sp, ap, mut fp) = wav2world(&data, fs as i32);
	main_menu();
	let orig_f0            = f0.clone();
	let orig_fp            = fp;
	let orig_sp            = sp.clone();
	let mut f0_ratio       = 1.0;
	let mut speed          = fp;
	let mut spectral_ratio = 1.0;
	let mut volume         = 1.0;
	let mut robot_flag     = false;
	let mut female_flag    = false;
	let mut mosaic_flag    = false;
	loop {
	    keys = device_state.get_keys();
	    if keys == vec![Keycode::P] { // change pitch
		pitch_menu();
		println!("\nf0 ratio: {:.1}", f0_ratio);
		loop {
		    keys = device_state.get_keys();
		    if keys == vec![Keycode::W] {
			f0_ratio += 0.1;
			println!("\nf0 ratio: {:.1}", f0_ratio);
		    } else if keys == vec![Keycode::S] {
			if f0_ratio >= 0.1 {
			    f0_ratio -= 0.1;
			}
			println!("\nf0 ratio: {:.1}", f0_ratio);
		    } else if keys == vec![Keycode::Enter] {
			change_pitch(&mut f0, f0_ratio);
			break;
		    }
		    std::thread::sleep(std::time::Duration::from_millis(100));
		}
		main_menu();
	    } else if keys == vec![Keycode::O] { // change speed
		speed_menu();
		println!("\nspeed: {:.1}", speed);
		loop {
		    keys = device_state.get_keys();
		    if keys == vec![Keycode::W] {
			speed += 0.1;
			println!("\nspeed: {:.1}", speed);
		    } else if keys == vec![Keycode::S] {
			if speed >= 3.1 {
			    speed -= 0.1;
			}
			println!("\nspeed: {:.1}", speed);
		    } else if keys == vec![Keycode::Enter] {
			change_speed(&mut fp, speed);
			break;
		    }
		    std::thread::sleep(std::time::Duration::from_millis(100));
		}
		main_menu();
	    } else if keys == vec![Keycode::I] { // change spectral envelope
		spectral_menu();
		println!("\nspectral envelope: {:.1}", spectral_ratio);
		loop {
		    keys = device_state.get_keys();
		    if keys == vec![Keycode::W] {
			spectral_ratio += 0.1;
			println!("\nspectral envelope: {:.1}", spectral_ratio);
		    } else if keys == vec![Keycode::S] {
			if spectral_ratio >= 0.1 {
			    spectral_ratio -= 0.1;
			}
			println!("\nspectral envelope: {:.1}", spectral_ratio);
		    } else if keys == vec![Keycode::Enter] {
			change_spectral_envelope(&mut sp, spectral_ratio);
			break;
		    }
		    std::thread::sleep(std::time::Duration::from_millis(100));
		}
		main_menu();
	    } else if keys == vec![Keycode::V] { // change volume
		volume_menu();
		println!("\nvolume: {:.1}", volume);
		loop {
		    keys = device_state.get_keys();
		    if keys == vec![Keycode::W] {
			volume += 0.1;
			change_spectral_envelope(&mut sp, spectral_ratio);
			println!("\nvolume: {:.1}", volume);
		    } else if keys == vec![Keycode::S] {
			if volume >= 0.1 {
			    volume -= 0.1;
			}
			println!("\nvolume: {:.1}", volume);
		    } else if keys == vec![Keycode::Enter] {
			break;
		    }
		    std::thread::sleep(std::time::Duration::from_millis(100));
		}
		main_menu();
	    } else if keys == vec![Keycode::Key1] { // Play
		let data: Vec<i16> = synthesis(&f0, &sp, &ap, fp, fs as i32).iter().map(|d| (*d * volume) as i16).collect();
		let source = SamplesBuffer::new(1, fs as u32, data).buffered();
		println!("\nPlaying...");
		sink.append(source.clone());
		sink.sleep_until_end();
		main_menu();
	    } else if keys == vec![Keycode::Key2] { // Robot
		if !robot_flag {
		    sp = orig_sp.clone();
	    	    to_robot(&mut f0);
		    robot_flag = true;
		    println!("\nRobot [On]");
		}
		main_menu();
	    } else if keys == vec![Keycode::Key3] { // Female
		if !female_flag {
		    sp = orig_sp.clone();
	    	    to_female(&mut f0, &mut sp);
		    female_flag = true;
		    println!("\nFemale [On]");
		}
		main_menu();
	    } else if keys == vec![Keycode::Key4] { // Mosaic
		if !mosaic_flag {
		    sp = orig_sp.clone();
	    	    to_mosaic(&mut f0, &mut sp);
		    mosaic_flag = true;
		    println!("\nMosaic [On]");
		}
		main_menu();
	    } else if keys == vec![Keycode::R] { // Reset
		println!("\nReset");
		f0 = orig_f0.clone();
		fp = orig_fp;
		sp = orig_sp.clone();
		robot_flag  = false;
		female_flag = false;
		mosaic_flag = false;
		main_menu();
	    } else if keys == vec![Keycode::Escape] { // Record
		break;
	    }
	}

	// let data: Vec<(f64, f64)> = source.clone().enumerate().map(|(i, x)| (i as f64, x as f64)).collect();
	// draw_spectrum(&data, "line.svg").expect("Failed draw graph");
	println!("\nStart with [Space] key...");
    }
}
