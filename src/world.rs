#![allow(dead_code)]

pub use Rust_WORLD::rsworld::synthesis;

use Rust_WORLD::rsworld::{
    cheaptrick,
    dio,
    d4c,
    harvest,
    stonemask,
};
use Rust_WORLD::rsworld_sys::{
    CheapTrickOption,
    DioOption,
    D4COption,
    HarvestOption,
};

pub fn change_pitch(f0: &mut Vec<f64>, rate: f64) {
    for x in f0.iter_mut() {
	*x *= rate;
    }
}

pub fn change_speed(fp: &mut f64, speed: f64) {
    if 3.0 <= speed {
	*fp = speed;
    }
}

pub fn change_spectral_envelope(sp: &mut Vec<Vec<f64>>, ratio: f64) {
    let tmp_sp = sp.clone();
    for (i, y) in sp.iter_mut().enumerate() {
	for (j, x) in y.iter_mut().enumerate() {
	    let idx = (j as f64/ratio) as usize;
	    if idx<tmp_sp[0].len() {
		*x = tmp_sp[i][idx];
	    }
	}
    }
}

pub fn to_robot(f0: &mut Vec<f64>) {
    for x in f0.iter_mut() {
	*x = 100.0;
    }
}

pub fn to_female(f0: &mut Vec<f64>, sp: &mut Vec<Vec<f64>>) {
    for x in f0.iter_mut() {
    	*x *= 2.5;
    }
    let tmp_sp = sp.clone();
    for (i, y) in sp.iter_mut().enumerate() {
	for (j, x) in y.iter_mut().enumerate() {
	    let idx = (j as f64/1.2) as usize;
	    *x = tmp_sp[i][idx];
	}
    }
}

pub fn to_mosaic(f0: &mut Vec<f64>, sp: &mut Vec<Vec<f64>>) {
    for x in f0.iter_mut() {
    	*x *= 0.5;
    }
    let tmp_sp = sp.clone();
    for (i, y) in sp.iter_mut().enumerate() {
	for (j, x) in y.iter_mut().enumerate() {
	    let idx = (j as f64/0.5) as usize;
	    if idx<tmp_sp[0].len() {
		*x = tmp_sp[i][idx];
	    }
	}
    }
}

pub fn wav2world(x: &Vec<f64>, fs: i32) -> (Vec<f64>, Vec<Vec<f64>>, Vec<Vec<f64>>, f64) {
    let option = HarvestOption::new();
    let (temporal_positions, f0) = harvest(&x, fs, &option);
    // let option = DioOption::new();
    // let (temporal_positions, f0) = dio(&x, fs, &option);
    let frame_period = option.frame_period;
    let f0           = stonemask(&x, fs, &temporal_positions, &f0);
    let mut option   = CheapTrickOption::new(fs);
    let spectrogram  = cheaptrick(&x, fs, &temporal_positions, &f0, &mut option);
    let option       = D4COption::new();
    let aperiodicity = d4c(&x, fs, &temporal_positions, &f0, &option);
    (f0, spectrogram, aperiodicity, frame_period)
}
