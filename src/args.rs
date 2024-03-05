use clap::Parser;

#[derive(Parser,Debug,Clone)]
#[command(version,about,author)]
pub struct Porcel8ProgramArgs {
    #[arg(short,long,help = "Filename of ROM to load.")]
    pub filename:Option<String>
}