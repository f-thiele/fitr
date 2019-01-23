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

use chrono::{DateTime, Utc};

mod util;

extern crate gpx;
extern crate gpxalyzer;

use std::io::BufReader;
use std::fs::File;
use itertools::izip;

use gpx::read;
use gpx::{Gpx, Track, TrackSegment};

#[macro_use] extern crate log;
use simplelog::*;

fn gpx_stuff() -> (std::vec::Vec<f64>, std::vec::Vec<DateTime<Utc>>) {
    // This XML file actually exists — try it for yourself!
    let file = File::open("tests/fixtures/example.gpx").unwrap();
    let reader = BufReader::new(file);

    // read takes any io::Read and gives a Result<Gpx, Error>.
    let gpx: Gpx = read(reader).unwrap();

    // Each GPX file has multiple "tracks", this takes the first one.
    let track: &Track = &gpx.tracks[0];
    // assert_eq!(track.name, Some(String::from("Example GPX Document")));

    // Each track will have different segments full of waypoints, where a
    // waypoint contains info like latitude, longitude, and elevation.
    let segment: &TrackSegment = &track.segments[0];

    // This is an example of retrieving the elevation (in meters) at certain points.

    gpxalyzer::decorate_speed(segment).unwrap();

    let mut elev = std::vec::Vec::new();
    let mut time = std::vec::Vec::new();

    for n in &segment.points {
        elev.push(n.elevation.unwrap());
        time.push(n.time.unwrap());
    }

    return (elev, time);
}
struct SigApp {
    data1: Vec<(f64, f64)>,
    data2: Vec<(f64, f64)>,
    y_range: [f64; 2],
    window: [f64; 2],
}

impl SigApp {
    fn new() -> SigApp {
        let (elev, time) = gpx_stuff();
        let mut data1 = std::vec::Vec::new();
        let mut count: f64 = 0.;
        let mut y_min: f64 = 0.;
        let mut y_max: f64 = 0.;
        let starttime = time[0].time();

        for (y, x) in izip!(&elev, &time) {
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
        info!("This only appears in the log file: {:}", last_point);


        SigApp {
            data1,
            data2,
            y_range: [0.8*y_min, 1.2*y_max],
            window: [0.0, last_point],
        }
    }

    fn update(&mut self) {
    }
}

struct App {
    size: Rect,
    x: f64,
    y: f64,
    ball: Rect,
    playground: Rect,
    dir_x: bool,
    dir_y: bool,
}

impl App {
    fn new() -> App {
        App {
            size: Default::default(),
            x: 0.0,
            y: 0.0,
            ball: Rect::new(10, 30, 10, 10),
            playground: Rect::new(10, 10, 100, 100),
            dir_x: true,
            dir_y: true,
        }
    }

    fn update(&mut self) {
        if self.ball.left() < self.playground.left() || self.ball.right() > self.playground.right()
        {
            self.dir_x = !self.dir_x;
        }
        if self.ball.top() < self.playground.top() || self.ball.bottom() > self.playground.bottom()
        {
            self.dir_y = !self.dir_y;
        }
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
    let mut sigapp = SigApp::new();


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
                .block(Block::default().borders(Borders::ALL).title("Pong"))
                .paint(|ctx| {
                    ctx.draw(&Line {
                        x1: f64::from(app.ball.left()),
                        y1: f64::from(app.ball.top()),
                        x2: f64::from(app.ball.right()),
                        y2: f64::from(app.ball.top()),
                        color: Color::Yellow,
                    });
                }).x_bounds([10.0, 110.0])
                .y_bounds([10.0, 110.0])
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
                        .title("Elevation [m]")
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
                    app.y += 1.0;
                }
                Key::Up => {
                    app.y -= 1.0;
                }
                Key::Right => {
                    app.x += 1.0;
                }
                Key::Left => {
                    app.x -= 1.0;
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

