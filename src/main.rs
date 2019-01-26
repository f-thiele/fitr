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
use std::io::BufReader;
use std::fs::File;
use std::env;
use std::time::Duration;
use std::error::Error;

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

use itertools::izip;

use getopts::Options;

use gpx::read;
use gpx::{Gpx, Track, TrackSegment};
use geo_types::Point;

#[macro_use] extern crate log;
use simplelog::{LevelFilter, CombinedLogger, TermLogger, WriteLogger};

mod util;

extern crate gpx;
extern crate gpxalyzer;

struct GPX_Data {
    filename: String,
    gpx: Gpx,
    track: Track,
    segment: TrackSegment,
}

impl GPX_Data {
    fn new(filename: String) -> Result<GPX_Data, Box<Error>> {
        let file = File::open(filename.as_str())?;
        let reader = BufReader::new(file);

        // read takes any io::Read and gives a Result<Gpx, Error>.
        let gpx: Gpx = read(reader)?;

        // for first demo use only the first track found
        let track: Track = gpx.tracks[0].clone();

        // Each track will have different segments full of waypoints, where a
        // waypoint contains info like latitude, longitude, and elevation.
        let segment: TrackSegment = track.segments[0].clone();

        Ok(GPX_Data {
            filename,
            gpx,
            track: track,
            segment: segment,
        })
    }
}


struct DiagramApp {
    data1: Vec<(f64, f64)>,
    data2: Vec<(f64, f64)>,
    y_range: [f64; 2],
    window: [f64; 2],
}

impl DiagramApp {
    fn new(filename: String) -> Result<DiagramApp, Box<Error>> {
        let mut gpx = GPX_Data::new(filename)?;

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

        Ok(DiagramApp {
            data1,
            data2,
            y_range: [0.8*y_min, 1.2*y_max],
            window: [0.0, last_point],
        })
    }

    fn update(&mut self) {
        // leave this in for later scroling and updating
    }
}

struct RouteApp {
    size: Rect,
    data: std::vec::Vec<Point<f64>>,
    draw_area: [f64; 4],
    mv_up: i64,
    mv_left: i64,
    mv_up_d: f64,
    mv_left_d: f64,
}

impl RouteApp {
    fn new(filename: String) -> Result<RouteApp, Box<Error>> {
        let gpx = GPX_Data::new(filename)?;
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

        Ok(RouteApp {
            size: Default::default(),
            data: points,
            draw_area: [x_range[0], y_range[0], x_range[1], y_range[1]],
            mv_up: 0,        // do not store any remaining scroll steps as default
            mv_left: 0,      // do not store any remaining scroll steps as default
            mv_up_d: 0.01,    // default: 10% movement in y-axis direction of visible region
            mv_left_d: 0.01,  // default: 10% movement in x-axis direction of visible region
        })
    }

    fn scroll_up(&mut self) {
        self.mv_up -= 1;
    }
    fn scroll_down(&mut self) {
        self.mv_up += 1;
    }
    fn scroll_left(&mut self) {
        self.mv_left += 1;
    }
    fn scroll_right(&mut self) {
        self.mv_left -= 1;
    }

    fn update(&mut self) {
        // measure visible distance along y-axis
        let y_visible_d = self.draw_area[3]-self.draw_area[1];

        // adjust top and bottom limit uniformly by scrolled steps and
        // defined distance increase
        self.draw_area[3] += y_visible_d*self.mv_up_d*self.mv_up as f64;
        self.draw_area[1] += y_visible_d*self.mv_up_d*self.mv_up as f64;

        // reset up/down movement counter
        self.mv_up = 0;

        // measure visible distance along y-axis
        let x_visible_d = self.draw_area[2]-self.draw_area[0];

        // adjust top and bottom limit uniformly by scrolled steps and
        // defined distance increase
        self.draw_area[2] += y_visible_d*self.mv_left_d*self.mv_left as f64;
        self.draw_area[0] += y_visible_d*self.mv_left_d*self.mv_left as f64;

        // reset up/down movement counter
        self.mv_left = 0;
    }
}

fn print_usage(program: &str, opts: Options) {
    println!("{}", opts.usage(&format!("Usage: {} <gpx-data-path>", program)));
}

fn main() {
    // log to terminal and file
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Warn, simplelog::Config::default()).unwrap(),
            WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), File::create("fitr.log").unwrap()),
        ]
    ).unwrap();

    // obtain arguments for running the program
    let args: Vec<String> = env::args().collect();

    let program = &args[0];

    let mut opts = Options::new();
    opts.optflag("h", "help", "Show this usage message.");

    let matches = match opts.parse(&args[1..]) {
        Ok(m)  => { m }
        Err(e) => { panic!(e.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let filename = if !matches.free.is_empty() {
        // if we have any matches left for we use the first one for the filename
        &matches.free[0]
    } else {
        // otherwise show help information
        print_usage(&program, opts);
        std::process::exit(1);
    };

    // return standard POSIX exit codes depending on how the run_prog routine
    // terminates
    ::std::process::exit(match run_prog(filename.to_string()) {
        Ok(_) => 0,
        Err(err) => {
            error!("Error while executing fitr. Error message: {:}", err);
            1
        }
    });
}


fn run_prog(filename: String) -> Result<(), Box<Error>> {
    // Terminal initialization
    let stdout = std::io::stdout().into_raw_mode()?;
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

    // 2D route app (scrollable and hence mutable)
    let mut route_app = RouteApp::new(filename.to_string())?;
    // diagram app of variable to show along time
    let diag_app = DiagramApp::new(filename.to_string())?;

    // main loop for showing TUI
    loop {
        let size = terminal.size()?;
        if size != route_app.size {
            terminal.resize(size)?;
            route_app.size = size;
        }

        terminal.draw(|mut f| {
            // split layout into two vertical parts of 50% each
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(route_app.size);

            // draw in the top part of the layout (chunks[0]) a tui widget canvas
            Canvas::default()
                .block(Block::default().borders(Borders::ALL).title("Route"))
                .paint(|ctx| {

                    // iterate now over all datapoints and draw a line from i to i+1
                    // until we reach the last one
                    for i in 0..(route_app.data.len()-2) {
                      ctx.draw(&Line {
                          x1: f64::from(route_app.data[i].lat()),
                          y1: f64::from(route_app.data[i].lng()),
                          x2: f64::from(route_app.data[i+1].lat()),
                          y2: f64::from(route_app.data[i+1].lng()),
                          color: Color::Yellow,
                      });
                    }
                }).x_bounds([route_app.draw_area[0], route_app.draw_area[2]])
                .y_bounds([route_app.draw_area[1], route_app.draw_area[3]])
                .render(&mut f, chunks[0]);

            // draw a tui widget chart in the bottom part of the layout (chunks[1])
            Chart::default()
                .block( //style and widget title
                    Block::default()
                        .title("Chart")
                        .title_style(Style::default().fg(Color::Cyan).modifier(Modifier::Bold))
                        .borders(Borders::ALL),
                )
                .x_axis( // x-axis label and dimension (we resize for now by factor of 60. for unit conversion to minutes)
                    Axis::default()
                        .title("Time [min]")
                        .style(Style::default().fg(Color::Gray))
                        .labels_style(Style::default().modifier(Modifier::Italic))
                        .bounds(diag_app.window)
                        .labels(&[
                            &format!("{}", diag_app.window[0] / 60.),
                            &format!("{}", (diag_app.window[0] + diag_app.window[1]) / 2.0 / 60.),
                            &format!("{}", diag_app.window[1] / 60.),
                        ]),
                )
                .y_axis( // y-axis label and ticks
                    Axis::default()
                        .title("Speed [m/s]")
                        .style(Style::default().fg(Color::Gray))
                        .labels_style(Style::default().modifier(Modifier::Italic))
                        .bounds(diag_app.y_range)
                        .labels(&[
                            &format!("{:.2}", diag_app.y_range[0]),
                            &format!("{:.2}", (diag_app.y_range[0] + diag_app.y_range[1]) / 2.0),
                            &format!("{:.2}", diag_app.y_range[1]),
                        ]),

                )
                .datasets(&[
                    Dataset::default()
                        .name("Testtrack")
                        .marker(Marker::Dot)
                        .style(Style::default().fg(Color::Cyan))
                        .data(&diag_app.data1), //use here the data1 saved for the diagram app
                ])
                .render(&mut f, chunks[1]);
        })?;

        // when in the main loop we react to key presses and leave upon pressing 'q'
        match events.next()? {
            util::Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Down => {
                    route_app.scroll_down();
                }
                Key::Up => {
                    route_app.scroll_up();
                }
                Key::Right => {
                    route_app.scroll_right();
                }
                Key::Left => {
                    route_app.scroll_left();
                }

                _ => {}
            },
            util::Event::Tick => {
                route_app.update();
            }
        }
    }

    Ok(())
}

