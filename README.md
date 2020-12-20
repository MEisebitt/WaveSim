# WaveSim: Simulating 2D waves on a hexagonal lattice
WaveSim is a small application that simulates and displays the nummerical solution of the 2D wave equation. It calculates the time evolution of the wave on a hexagonal lattice
in contrast to most simlations which utilize a square or rectangular grid. WaveSim allows for changing simulation parameters and initial conditions of the wave and for choosing
an arbitrary shape for the simulation.
## Prerequisites
- To compile the programm [Rust](https://www.rust-lang.org/learn/get-started "Install Rust") is nessecary
- On Linux [gtk+3](https://www.gtk.org/docs/installations/linux/ "GTK installation page") is required to compile the application
## Compilation
- Download repository
- Navigate into the WaveSim folder in a terminal and run ```cargo build --release```
## Execution
- After compilation run with ```cargo run --release```
- Alternatively run the executable file in WaveSim/target/release
## Screenshots
![Example screenshot](https://raw.githubusercontent.com/MEisebitt/WaveSim/main/screenshots/example_1.png)
