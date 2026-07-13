use crate::gameboy::GameBoy;

mod cpu;
mod gameboy;
mod memory;
mod ppu;
mod registers;

#[allow(unused_variables, unused_mut)]
fn main() {
    let args: Vec<_> = std::env::args().collect();
    dbg!(args);
    let mut gameboy = GameBoy::default();
    gameboy.run();

    println!("Hello, world!");
}
