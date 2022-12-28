# PlatiniumOS
#### Thank you, Philipp Oppermann for an amazing tutorial on how to write an OS in Rust!

PlatiniumOS is an operating system written in Rust. I use it to learn how operating
systems work. I followed [blog_os](https://os.phil-opp.com/) tutorial while writing it,
but I added a few features like a shell. I want it to be easily expandable (at this moment, a lot of code
related to keyboard input is tightly coupled with the input handler, which doesn't scale well)
and have support for networking and files.

## Goals
[ ] Networking

[ ] Modularity

[ ] Filesystem

[ ] Shell

[ ] Hier support

## Running
To run PlatiniumOS you need [Rust toolchain](https://www.rust-lang.org/tools/install). 

First, you need to install dependencies using `cargo install`.

Now you can run it with `cargo run`.

Because it is majorly from Philipp's tutorial, you can check his tutorial for more information.

## Shell
Shell currently support only a few commands: `help`, `color`, `set`, `calc`, `version`. You
can learn more about them using `help` command. Shell also support history (you move through it using
arrow keys). Variables can be referred using `$` sign, for example `$var`. Using `$()` you can interpolate
output of other command inside a command. For example, `echo $(calc 2 + 2)` will print `4`.