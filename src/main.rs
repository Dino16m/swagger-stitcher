mod stitcher;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// File containing the base spec
    #[clap(short, long)]
    base: String,

    /// Comma seperated list of paths or filenames to exclude
    #[clap(short, long, default_value = "")]
    exclude: String,

    ///Documentation Directory.
    #[clap(short, long, default_value = "./")]
    directory: String,

    ///Output file. Will be created if it doesn't exist
    #[clap(short, long, default_value = "./output.yaml")]
    output: String,
}

fn main() {
    let args: Args = Args::parse();
    let exclusion_list: Vec<&str> = args.exclude.split(",").collect();
    let options = stitcher::StitcherOptions {
        base: args.base,
        directory: args.directory,
        output: args.output,
        exclude: exclusion_list.iter().map(|&val| val.to_string()).collect(),
    };

    stitcher::stitch(options);
}
