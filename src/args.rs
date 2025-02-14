use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, about, author)]
pub struct Porcel8ProgramArgs {
    /// CHIP-8 rom file to load
    pub filename: String,
    #[arg(short, long, help = "Draw scale of window", default_value_t = 8f32)]
    pub draw_scale: f32,
    #[arg(
        short,
        long,
        help = "Emulate new behaviour of instructions (As seen in Chip-48 and SuperChip8)",
        default_value_t = true
    )]
    pub new_chip8_behaviour: bool,
    #[arg(
        short='i',
        long,
        help = "Halt on invalid instruction",
        default_value_t = false
    )]
    pub halt_on_invalid: bool,
}
