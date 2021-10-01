Some code for solving a Sudoku puzzle, mostly not using recursion.

I wrote this as an exercise when learning Rust.

Desired functionality:

* Read input on stdin, write formatted solution(s) on stdout.
* Make as many solutions as desired by user.
* Handle Sudoku puzzle of sizes 0x0 up to 16x16 at least.
* The solver shall return the solutions using the Rust
  [Iterator trait](https://doc.rust-lang.org/std/iter/index.html).

Also I wanted to experiment with putting some code up on GitHub.

Code tries to use terminology from [Wikipedia page on
Glossary of Sudoku](https://en.wikipedia.org/wiki/Glossary_of_Sudoku).

Some inspiration was fetched from [a Computerphile video about Sudoku
solver code](https://youtu.be/G_UYXzGuqvM).

To solve a puzzle using the provided `sudoku-solver` binary pipe it in to stdin:
```shell
cargo run --bin sudoku-solve < samples/easy.sudoku
```

By default it will output the first solution it finds and then stop. If
you want more solutions provide the maximum number of solutions you want as
an argument:
```shell
cargo run --bin sudoku-solve -- 42 < samples/with-many-solutions.sudoku
```

