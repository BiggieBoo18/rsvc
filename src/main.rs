use std::fs::File;
use std::io;
use rodio::source::Source;
use plotlib::page::Page;
use plotlib::view::ContinuousView;
use plotlib::style::Line;
use failure::Error;

fn draw_spectrum(data: &Vec<(f64, f64)>, outpath: &str) -> Result<(), Error> {
    let mut style = plotlib::line::Style::new();
    let li = plotlib::line::Line::new(&data)
        .style(style.colour("#000000"));
    let v = ContinuousView::new().add(&li);
    Page::single(&v).save(outpath)?;
    Ok(())
}


fn main() {
    let device = rodio::default_output_device().unwrap();
    let sink = rodio::Sink::new(&device);

    let file = File::open("sample_audio/sample2.wav").unwrap();
    let source  = rodio::Decoder::new(io::BufReader::new(file)).unwrap().buffered();

    let data: Vec<(f64, f64)> = source.clone().enumerate().map(|(i, x)| (i as f64, x as f64)).collect();
    draw_spectrum(&data, "line.svg").expect("Failed draw graph");

    println!("{}", source.sample_rate()); // 22050

    sink.append(source.clone());
    sink.sleep_until_end();
}
