extern crate anyhow;
extern crate cpal;
extern crate hound;

use std::fs::File;
use std::io;
use rodio::source::Source;
use rodio::buffer::SamplesBuffer;
use plotlib::page::Page;
use plotlib::view::ContinuousView;
use plotlib::style::Line;
use failure::Error;
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use device_query::{DeviceQuery, DeviceState, Keycode};
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

#[allow(dead_code)]
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
fn change_spectral_envelope(sp: &mut Vec<Vec<f64>>, ratio: f64) {
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

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    match format {
        cpal::SampleFormat::U16 => hound::SampleFormat::Int,
        cpal::SampleFormat::I16 => hound::SampleFormat::Int,
        cpal::SampleFormat::F32 => hound::SampleFormat::Float,
    }
}

fn wav_spec_from_format(format: &cpal::Format) -> hound::WavSpec {
    hound::WavSpec {
        channels: format.channels as _,
        sample_rate: format.sample_rate.0 as _,
        bits_per_sample: (format.data_type.sample_size() * 8) as _,
        sample_format: sample_format(format.data_type),
    }
}

fn record(path: &str, sec: u64) -> Result<(), anyhow::Error> {
    // ref: cpal example(record_wav.rs)
    // Use the default host for working with audio devices.
    let host = cpal::default_host();

    // Setup the default input device and stream with the default input format.
    let device = host.default_input_device().expect("Failed to get default input device");
    // println!("Default input device: {}", device.name()?);
    let mut format = device.default_input_format().expect("Failed to get default input format");
    format.channels  = 2;
    format.data_type = cpal::SampleFormat::I16;
    // println!("Default input format: {:?}", format);
    let event_loop = host.event_loop();
    let stream_id = event_loop.build_input_stream(&device, &format)?;
    event_loop.play_stream(stream_id)?;

    // The WAV file we're recording to.
    let spec = wav_spec_from_format(&format);
    let writer = hound::WavWriter::create(path, spec)?;
    let writer = std::sync::Arc::new(std::sync::Mutex::new(Some(writer)));

    // A flag to indicate that recording is in progress.
    println!("\nBegin recording for {} sec...", sec);
    let recording = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));

    // Run the input stream on a separate thread.
    let writer_2 = writer.clone();
    let recording_2 = recording.clone();
    std::thread::spawn(move || {
        event_loop.run(move |id, event| {
            let data = match event {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("an error occurred on stream {:?}: {}", id, err);
                    return;
                }
            };

            // If we're done recording, return early.
            if !recording_2.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }
            // Otherwise write to the wav writer.
            match data {
                cpal::StreamData::Input { buffer: cpal::UnknownTypeInputBuffer::I16(buffer) } => {
                    if let Ok(mut guard) = writer_2.try_lock() {
                        if let Some(writer) = guard.as_mut() {
                            for &sample in buffer.iter() {
                                writer.write_sample(sample).ok();
                            }
                        }
                    }
                },
                _ => (),
            }
        });
    });

    // Let recording go for roughly three seconds.
    std::thread::sleep(std::time::Duration::from_secs(sec));
    recording.store(false, std::sync::atomic::Ordering::Relaxed);
    writer.lock().unwrap().take().unwrap().finalize()?;
    println!("Recording {} complete!", path);
    Ok(())
}

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

	let data: Vec<f64> = source.clone().map(|d| d as f64).step_by(2).collect();
	#[allow(unused_mut)]
	let (mut f0, mut sp, ap, mut fp) = wav2world(&data, fs as i32);
	// change_pitch(&mut f0, 0.5);
	// change_speed(&mut fp, 3.5);
	// change_spectral_envelope(&mut sp, 0.5);
	println!("[R]: Robot\n[F]: Female\n[M]: Mosic");
	let mut volume = 1.0;
	loop {
	    keys = device_state.get_keys();
	    if keys.contains(&Keycode::R) {
		to_robot(&mut f0);
		volume = 2.0;
		break;
	    } else if keys.contains(&Keycode::F) {
		to_female(&mut f0, &mut sp);
		break;
	    } else if keys.contains(&Keycode::M) {
		to_mosaic(&mut f0, &mut sp);
		volume = 4.0;
		break;
	    }
	}
	let data: Vec<i16> = synthesis(&f0, &sp, &ap, fp, fs as i32).iter().map(|d| (*d * volume) as i16).collect();
	let source = SamplesBuffer::new(1, fs as u32, data).buffered();

	// let data: Vec<(f64, f64)> = source.clone().enumerate().map(|(i, x)| (i as f64, x as f64)).collect();
	// draw_spectrum(&data, "line.svg").expect("Failed draw graph");

	sink.append(source.clone());
	sink.sleep_until_end();
	println!("\nStart with [Space] key...");
    }
}
