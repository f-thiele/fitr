fitr
============

![fitr output](/assets/demo.jpg)

fitr is a GPX-file track analysis program with a command line interface written in rust.

## Work in progress

This project is pretty much a work in progress. Features that are covered so far:

 - show 2D representation of a GPS track (see screenshot) in the terminal
 - calculate speed profile of GPS track points and plot it in the terminal
 - show elevation, longitude, lattitude profiles (not yet configurable but need to change code manually)

The file used needs to be specified by the user as a command line argument (example file not packaged here).

## What should it be?

There are currently three goals for this program

- make 2D representation of GPX tracks
- let it be "playable" and show progress bar as well as location indicator
- include all kinds of statistics: speed, elevation, distance etc. that are on-the-fly switchable for the user

## Installation

After cloning the github project and its used library [gpxalyzer](https://github.com/f-thiele/gpxalyzer) into
the same directory a simple `cargo run` should pull further dependencies and run the TUI.

First command line argument should be the gpx-data-path and `-h` shows the usage message.

## License
This project is licensed under the terms of the GPL v3 or any later version (**GPL-3.0-or-later**).

fitr Copyright (C) 2019 **Fabian A.J. Thiele**, <fabian.thiele@posteo.de>
