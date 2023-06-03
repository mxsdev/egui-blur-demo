use program::Program;

mod context;
mod pipeline;
mod program;
mod surface;
mod ui;
mod util;
mod window_texture;

fn main() {
    let program = pollster::block_on(Program::new());
    program.run();
}
