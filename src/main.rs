mod cpu;
mod memory;
mod registers;

#[allow(unused_variables)]
fn main() {
    let args: Vec<_> = std::env::args().collect();
    dbg!(args);
    let cpu = cpu::Cpu::default();
    println!("Hello, world!");
}
