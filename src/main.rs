use backrub::program;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "backrub", about = "A deduplicating backup program")]
enum Opts {
    Init(InitOps),
    Create(CreateOpts),
    Restore(RestoreOpts),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "init", about = "Initialize a new repository instance")]
struct InitOps {
    /// The path to init as a repository
    repository: String,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "create",
    about = "Create a new backup instance in the repository"
)]
struct CreateOpts {
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

#[derive(Debug, StructOpt)]
#[structopt(name = "restore", about = "Restore information in a backup instance")]
struct RestoreOpts {
    #[structopt(short, long)]
    /// Activate debug mode
    debug: bool,
    /// The repository to write the backup to
    repository: String,
    /// The name under which to store the backup
    name: String,

    /// The path to backup
    path: String,
}

fn main() {
    let options = Opts::from_args();
    let program_result = match options {
        Opts::Init(opts) => program::initialize_repository(&opts.repository),
        Opts::Create(opts) => program::make_backup(&opts.repository, &opts.path, &opts.name),
        Opts::Restore(opts) => program::restore_backup(&opts.repository, &opts.path, &opts.name),
    };
    match program_result {
        Ok(_) => (),
        Err(e) => println!("Operation failed. Reason: {}", e),
    }
}
