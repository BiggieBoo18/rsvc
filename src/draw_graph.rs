#![allow(dead_code)]

use plotlib::page::Page;
use plotlib::view::ContinuousView;
use plotlib::style::Line;
use failure::Error;

pub fn draw_spectrum(data: &Vec<(f64, f64)>, outpath: &str) -> Result<(), Error> {
    let mut style = plotlib::line::Style::new();
    let li = plotlib::line::Line::new(&data)
        .style(style.colour("#000000"));
    let v = ContinuousView::new().add(&li);
    Page::single(&v).save(outpath)?;
    Ok(())
}
