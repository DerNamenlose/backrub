use std::thread;
use structopt::StructOpt;
use walkdir::WalkDir;

mod fssource;
mod repository;

#[derive(Debug, StructOpt)]
#[structopt(name = "backrub", about = "A deduplicating backup program")]
/// The command line arguments for this program
struct Opts {
    #[structopt(short, long)]
    /// Activate debug mode
    debug: bool,

    /// The path to backup
    path: String,

    /// The repository to write the backup to
    repository: String,

    /// The name under which to store the backup
    name: String,
}

fn main() {
    let opts = Opts::from_args();

    repository::initialize(&opts.repository);

    repository::add_block(&opts.repository, "This is a test".as_bytes());

    // let fssource = fssource::start(opts.path);

    // fssource::stop(fssource);
}
