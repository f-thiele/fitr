fitr
============

fitr is a GPX track analysis program with a command line interface written in rust.

## Work in progress

This is currently a placeholder of work in progress associated with the project. Technically
it means this program doesn't do anything! Well technically it loads a GPX file (not packaged here)
and plots for one track segment the Elevation versus the Time spent as for instance useful for a
runners analysis.

## What should it be?

There are currently three goals for this program

- make 2D representation of GPX tracks
- let it be "playable" and show progress bar as well as location indicator
- include all kinds of statistics: speed, elevation, distance etc.

## Installation

After cloning the github project and its used library [gpxalyzer](https://github.com/f-thiele/gpxalyzer) into
the same directory a simple `cargo run` should pull further dependencies and run the TUI.

## License
This project is licensed under the terms of the GPL v3 or any later version (**GPL-3.0-or-later**).

fitr Copyright (C) 2019 **Fabian A.J. Thiele**, <fabian.thiele@posteo.de>
