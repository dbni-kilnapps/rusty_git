
use clap::Parser;
mod init;
mod add;
mod commit;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli{
    command: String,
    args: Vec<String>,
    //long arg
    // #[arg(short, long)]
    // verbose: bool,
}

fn main() {
    let args = Cli::parse();
    //match statment for arguments
    match args.command.as_str() {
        "init" => {
            init::init();
        },
        "add" => {
            add::add(args.args[0].clone());
        },
        _ => {
            println!("Invalid command");
        }
    }
    

}
