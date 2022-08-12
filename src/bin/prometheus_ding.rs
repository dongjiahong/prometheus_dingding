use anyhow::Result;
use clap::{Parser, Subcommand};
use ding::utils;

#[derive(Parser)]
#[clap(author = "jiahong", version = utils::version::get_version())]
struct Args {
    #[clap(short, long)]
    port: String,
}

#[tokio::main]
async fn main() ->Result<()> {
    Ok(())
}
