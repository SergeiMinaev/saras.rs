use futures_lite::future;
use saras::default_app;
use clap::{Parser, Subcommand};
use saras::auth::hashing;


#[derive(Subcommand, Debug)]
enum Commands {
    HashPwd { pwd: String },
}


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    commands: Option<Commands>,
}


fn main() {
	let cli = Cli::parse();

    match &cli.commands {
        Some(Commands::HashPwd { pwd}) => {
			let hash = hashing::hash_pwd(pwd);
			println!("hash: {hash}");
        }
        None => {
            println!("Running default app.");
			future::block_on(default_app::run());
        }
    }
}
