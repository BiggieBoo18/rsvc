use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};

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

pub fn record(path: &str, sec: u64) -> Result<(), anyhow::Error> {
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
