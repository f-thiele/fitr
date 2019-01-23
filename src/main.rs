//  fitr  --  GPX track analysis for the command line with rust
//  Copyright (C) 2019 - Fabian A.J. Thiele, <fabian.thiele@posteo.de>
//
//  This file is part of fitr.
//
//  fitr is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  fitr is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <https://www.gnu.org/licenses/>.
use std::io;
use std::time::Duration;

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::canvas::{Canvas, Line};
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Widget};
use tui::Terminal;

mod util;

extern crate gpx;
extern crate gpxalyzer;

use std::io::BufReader;
use std::fs::File;
use itertools::izip;

use gpx::read;
use gpx::{Gpx, Track, TrackSegment};
use geo_types::Point;

#[macro_use] extern crate log;
use simplelog::*;


struct GPX_Data {
    filename: String,
    gpx: Gpx,
    track: Track,
    segment: TrackSegment,
}

impl GPX_Data {
    fn new(filename: String) -> GPX_Data {
        let file = File::open(filename.as_str()).unwrap();
        let reader = BufReader::new(file);

        // read takes any io::Read and gives a Result<Gpx, Error>.
        let gpx: Gpx = read(reader).unwrap();

        // Each GPX file has multiple "tracks", this takes the first one.
        let track: Track = gpx.tracks[0].clone();
        // assert_eq!(track.name, Some(String::from("Example GPX Document")));

        // Each track will have different segments full of waypoints, where a
        // waypoint contains info like latitude, longitude, and elevation.
        let segment: TrackSegment = track.segments[0].clone();

        GPX_Data {
            filename,
            gpx,
            track: track,
            segment: segment,
        }
    }
}


struct SigApp {
    data1: Vec<(f64, f64)>,
    data2: Vec<(f64, f64)>,
    y_range: [f64; 2],
    window: [f64; 2],
}

impl SigApp {
    fn new() -> SigApp {
        let mut gpx = GPX_Data::new("tests/fixtures/example.gpx".to_string());

        gpxalyzer::decorate_speed(&mut gpx.segment);
        let yquant = gpxalyzer::get_speed(&gpx.segment);
        let time = gpxalyzer::get_time(&gpx.segment);
        let mut data1 = std::vec::Vec::new();
        let mut y_min: f64 = 0.;
        let mut y_max: f64 = 0.;
        let starttime = time[0].time();

        for (y, x) in izip!(&yquant, &time) {
            let duration = x.time().signed_duration_since(starttime);
            data1.push((duration.num_seconds() as f64, *y));

            if y > &y_max {
                y_max = *y;
            }
            if y < &y_min {
                y_min = *y;
            }
        }
        let data2 = data1.clone();

        let last_point = time[time.len()-1].time().signed_duration_since(starttime).num_seconds() as f64;

        SigApp {
            data1,
            data2,
            y_range: [0.8*y_min, 1.2*y_max],
            window: [0.0, last_point],
        }
    }

    fn update(&mut self) {
        // leave this in for later scroling and updating
    }
}

struct App {
    size: Rect,
    data: std::vec::Vec<Point<f64>>,
    playground: [f64; 4],
}

impl App {
    fn new() -> App {
        let gpx = GPX_Data::new("tests/fixtures/example.gpx".to_string());
        let mut points: std::vec::Vec<Point<f64>> = std::vec::Vec::new();
        for p in &gpx.segment.points {
            points.push(p.point());
        }
        let mut x_range: [f64; 2] = gpxalyzer::get_range_lattitude(&gpx.segment);
        info!("x-range {} to {}", x_range[0], x_range[1]);
        let mut y_range: [f64; 2] = gpxalyzer::get_range_longitude(&gpx.segment);
        info!("y-range {} to {}", y_range[0], y_range[1]);

        // multiply with safety margin of 0.25 distance
        let margin_factor = 0.25;
        let x_dist = x_range[1]-x_range[0];
        let y_dist = y_range[1]-y_range[0];

        x_range[0] -= x_dist*margin_factor;
        x_range[1] += x_dist*margin_factor;
        y_range[0] -= y_dist*margin_factor;
        y_range[1] += y_dist*margin_factor;

        App {
            size: Default::default(),
            data: points,
            playground: [x_range[0], y_range[0], x_range[1], y_range[1]],
        }
    }

    fn update(&mut self) {
        // leave this in for later scroling and updating
    }
}

fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Warn, simplelog::Config::default()).unwrap(),
            WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), File::create("fitr.log").unwrap()),
        ]
    ).unwrap();

    ::std::process::exit(match run_prog() {
        Ok(_) => 0,
        Err(err) => {
            error!("error: {:?}", err);
            1
        }
    });
}


fn run_prog() -> Result<(), failure::Error> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // Setup event handlers
    let config = util::Config {
        tick_rate: Duration::from_millis(100),
        ..Default::default()
    };
    let events = util::Events::with_config(config);

    // App
    let mut app = App::new();
    // App
    let sigapp = SigApp::new();

    loop {
        let size = terminal.size()?;
        if size != app.size {
            terminal.resize(size)?;
            app.size = size;
        }

        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(app.size);
            Canvas::default()
                .block(Block::default().borders(Borders::ALL).title("Route"))
                .paint(|ctx| {
                    for i in 0..(app.data.len()-2) {
                      ctx.draw(&Line {
                          x1: f64::from(app.data[i].lat()),
                          y1: f64::from(app.data[i].lng()),
                          x2: f64::from(app.data[i+1].lat()),
                          y2: f64::from(app.data[i+1].lng()),
                          color: Color::Yellow,
                      });
                    }
                }).x_bounds([app.playground[0], app.playground[2]])
                .y_bounds([app.playground[1], app.playground[3]])
                .render(&mut f, chunks[0]);

            Chart::default()
                .block(
                    Block::default()
                        .title("Chart")
                        .title_style(Style::default().fg(Color::Cyan).modifier(Modifier::Bold))
                        .borders(Borders::ALL),
                )
                .x_axis(
                    Axis::default()
                        .title("Time [min]")
                        .style(Style::default().fg(Color::Gray))
                        .labels_style(Style::default().modifier(Modifier::Italic))
                        .bounds(sigapp.window)
                        .labels(&[
                            &format!("{}", sigapp.window[0] / 60.),
                            &format!("{}", (sigapp.window[0] + sigapp.window[1]) / 2.0 / 60.),
                            &format!("{}", sigapp.window[1] / 60.),
                        ]),
                )
                .y_axis(
                    Axis::default()
                        .title("Speed [m/s]")
                        .style(Style::default().fg(Color::Gray))
                        .labels_style(Style::default().modifier(Modifier::Italic))
                        .bounds(sigapp.y_range)
                        .labels(&[
                            &format!("{:.2}", sigapp.y_range[0]),
                            &format!("{:.2}", (sigapp.y_range[0] + sigapp.y_range[1]) / 2.0),
                            &format!("{:.2}", sigapp.y_range[1]),
                        ]),

                )
                .datasets(&[
                    Dataset::default()
                        .name("Testtrack")
                        .marker(Marker::Dot)
                        .style(Style::default().fg(Color::Cyan))
                        .data(&sigapp.data1),
                    // Dataset::default()
                    //     .name("data3")
                    //     .marker(Marker::Braille)
                    //     .style(Style::default().fg(Color::Yellow))
                    //     .data(&sigapp.data2),
                ])
                .render(&mut f, chunks[1]);
        })?;

        match events.next()? {
            util::Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Down => {
                    //app.y += 1.0;
                }
                Key::Up => {
                    //app.y -= 1.0;
                }
                Key::Right => {
                    //app.x += 1.0;
                }
                Key::Left => {
                    //app.x -= 1.0;
                }

                _ => {}
            },
            util::Event::Tick => {
                app.update();
            }
        }
    }

    Ok(())
}

