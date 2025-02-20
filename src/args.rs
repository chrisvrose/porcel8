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
    /// Use updated CHIP-8 behaviours.
    pub new_chip8_behaviour: bool,
    #[arg(
        short='i',
        long,
        help = "Halt on invalid instruction",
        default_value_t = false
    )]
    /// Halt on finding invalid instruction
    pub halt_on_invalid: bool,
    /// Enable Instruction Throttling 
    #[arg(short='t', default_value_t=true)]
    pub do_instruction_throttling: bool,
    /// Target Instructions per second, if throttling is enabled
    #[arg(short='r',long,default_value_t=750u64)]
    pub ips_throttling_rate: u64
}
