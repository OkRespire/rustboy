mod cpu;
mod memory;
mod registers;

#[allow(unused_variables, unused_mut)]
fn main() {
    let args: Vec<_> = std::env::args().collect();
    dbg!(args);
    let mut cpu = cpu::Cpu::default();

    println!("Hello, world!");
}
