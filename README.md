# rrun

Run your rust source file conveniently (just like `go run`).

## USAGE

```
$ rrun --help

Rust Runner 0.1
nextzhou <nextzhou@gmail.com>

USAGE:
    rrun [input] [-- <args>...]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <input>      main source file, execute 'cargo run' if empty
    <args>...    arguments of your program
```

Run a single source file:

```
$ cat hello.rs
fn main() {
    println!("Hello Rrun.");
}

$ rrun hello.rs     # or 'rrun hello', without ".rs" extension
Hello Rrun.
```


Run a single source file with arguments:

```
$ cat sum.rs       # print the sum of command arguments number
fn main() {
    let sum: i32 = std::env::args().map(|s| s.parse::<i32>().unwrap_or_default()).sum();
    println!("{}", sum);
}

$ rrun sum -- 3 4 5    # just like `rustc sum.rs && ./sum 3 4 5`
12
```

Run a cargo project (in project directory):

```
$ rrun          # This has the same behavior as the `cargo run` command
```
