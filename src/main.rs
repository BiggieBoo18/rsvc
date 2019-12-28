use std::fs::File;
use std::io;
use rodio::source::Source;
use rodio::buffer::SamplesBuffer;
use plotlib::page::Page;
use plotlib::view::ContinuousView;
use plotlib::style::Line;
use failure::Error;
#[allow(unused_imports)]
use Rust_WORLD::rsworld::{
    cheaptrick,
    dio,
    d4c,
    harvest,
    stonemask,
    synthesis,
};
#[allow(unused_imports)]
use Rust_WORLD::rsworld_sys::{
    CheapTrickOption,
    DioOption,
    D4COption,
    HarvestOption,
};

fn draw_spectrum(data: &Vec<(f64, f64)>, outpath: &str) -> Result<(), Error> {
    let mut style = plotlib::line::Style::new();
    let li = plotlib::line::Line::new(&data)
        .style(style.colour("#000000"));
    let v = ContinuousView::new().add(&li);
    Page::single(&v).save(outpath)?;
    Ok(())
}

fn wav2world(x: &Vec<f64>, fs: i32) -> (Vec<f64>, Vec<Vec<f64>>, Vec<Vec<f64>>, f64) {
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

#[allow(dead_code)]
fn change_pitch(f0: &mut Vec<f64>, rate: f64) {
    for x in f0.iter_mut() {
	*x *= rate;
    }
}

#[allow(dead_code)]
fn to_robot(f0: &mut Vec<f64>) {
    for x in f0.iter_mut() {
	*x = 100.0;
    }
}

#[allow(dead_code)]
fn change_speed(fp: &mut f64, speed: f64) {
    if 3.0 <= speed {
	*fp = speed;
    }
}

#[allow(dead_code)]
fn to_female(f0: &mut Vec<f64>, sp: &mut Vec<Vec<f64>>) {
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

#[allow(dead_code)]
fn to_mosaic(f0: &mut Vec<f64>, sp: &mut Vec<Vec<f64>>) {
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

fn main() {
    let device = rodio::default_output_device().unwrap();
    let sink = rodio::Sink::new(&device);

    let file = File::open("sample_audio/wav0.wav").unwrap();
    let source  = rodio::Decoder::new(io::BufReader::new(file)).unwrap().buffered();
    let fs = source.sample_rate();

    let data: Vec<f64> = source.clone().map(|d| d as f64).collect();
    #[allow(unused_mut)]
    let (mut f0, mut sp, ap, mut fp) = wav2world(&data, fs as i32);
    // change_pitch(&mut f0, 4.0);
    // to_robot(&mut f0);
    // change_speed(&mut fp, 10.0);
    // to_female(&mut f0, &mut sp);
    // to_mosaic(&mut f0, &mut sp);
    let data: Vec<i16> = synthesis(&f0, &sp, &ap, fp, fs as i32).iter().map(|d| *d as i16).collect();
    let source = SamplesBuffer::new(1, fs as u32, data).buffered();
    
    let data: Vec<(f64, f64)> = source.clone().enumerate().map(|(i, x)| (i as f64, x as f64)).collect();
    draw_spectrum(&data, "line.svg").expect("Failed draw graph");


    sink.append(source.clone());
    sink.sleep_until_end();
}
