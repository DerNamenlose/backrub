use backrub::create;
use backrub::errors::Error;
use backrub::instances;
use backrub::program;
use backrub::restore;
use directories::ProjectDirs;
use std::path::Path;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "backrub", about = "A deduplicating backup program")]
enum Opts {
    Init(InitOps),
    Create(CreateOpts),
    Instances(InstancesOpts),
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
#[structopt(name = "instances", about = "List backup instances in the repository")]
struct InstancesOpts {
    #[structopt(short, long)]
    /// Activate debug mode
    debug: bool,
    /// The repository to write the backup to
    repository: String,
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

fn main() -> backrub::errors::Result<()> {
    let env = env_logger::Env::new().filter_or("BACKRUB_LOG", "info");
    env_logger::Builder::from_env(env).init();
    let options = Opts::from_args();
    let cache_dir = ProjectDirs::from("de", "geekbetrieb", "backrub")
        .map(|p| p.cache_dir().join("block_cache"))
        .ok_or(Error {
            message: "Could not calculate block cache directory",
            cause: None,
        })?;
    let program_result = match options {
        Opts::Init(opts) => program::initialize_repository(&opts.repository),
        Opts::Create(opts) => {
            create::make_backup(&opts.repository, &opts.path, &cache_dir, &opts.name)
        }
        Opts::Instances(opts) => instances::instances(&Path::new(&opts.repository)),
        Opts::Restore(opts) => restore::restore_backup(&opts.repository, &opts.path, &opts.name),
    };
    program_result
}
