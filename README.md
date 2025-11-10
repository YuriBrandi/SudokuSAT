# A super fast Sudoku SAT Solver built in Rust

This little project consists of a Rust-based egui app for solving Sudoku problems via SAT reduction.

<img width="750" alt="GUI screenshot" src="https://github.com/user-attachments/assets/eaa70336-f5c4-4973-9eb8-6e46c8041c36" />

#### What it can do:

- Generate random puzzles;
- Show DIMACS CNF representation for the given puzzle;
- Check the correctness of a solution;
- Solve by using a naive backtracking algorithm;
- Solve via varisat by reducing to a SAT problem;
- Measure time for both solvers;
- Work on matrices up to **25x25**†.

† *(limited for visibility reasons, can actually work for any size)*


### Binaries

Binaries are available for Windows, MacOS and Linux here: https://github.com/YuriBrandi/Sudoku-temp/releases

*Note: may not run in a virtualized Windows install if hardware acceleration is missing.*

### Bibliography

https://jix.github.io/varisat

https://people.sc.fsu.edu/~jburkardt/data/cnf/cnf.html

https://sat.inesc-id.pt/~ines/publications/aimath06.pdf


### License

This project is distributed under the [MIT license](LICENSE).
