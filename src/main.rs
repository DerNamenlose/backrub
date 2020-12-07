use backrub::create;
use backrub::errors::error;
use backrub::errors::Error;
use backrub::instances;
use backrub::program;
use backrub::restore;
use backrub::show;
use directories::ProjectDirs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "backrub", about = "A deduplicating backup program")]
enum Opts {
    Init(InitOps),
    Create(CreateOpts),
    Instances(InstancesOpts),
    Show(ShowOpts),
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
    #[structopt(short, long)]
    /// exclude files matching the given regex
    exlude: Option<Vec<String>>,
    #[structopt(long)]
    /// load the exclude expressions from a file
    exclude_from: Option<PathBuf>,
    #[structopt(short, long, required = true)]
    /// The path to backup
    sources: Vec<String>,
    #[structopt(short, long)]
    /// The repository to write the backup to
    repository: String,
    #[structopt(short, long)]
    /// The name under which to store the backup
    name: String,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "instances", about = "List backup instances in the repository")]
struct InstancesOpts {
    #[structopt(short, long)]
    /// Activate debug mode
    debug: bool,
    #[structopt(short, long)]
    /// The repository to list the instances from
    repository: String,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "show", about = "Show the details of a given instance")]
struct ShowOpts {
    #[structopt(short, long)]
    debug: bool,
    #[structopt(short, long)]
    /// Include the contents of the instance
    contents: bool,
    #[structopt(short, long)]
    /// The repository to retrieve the instance from
    repository: String,
    #[structopt(short, long)]
    /// The instance to retrieve
    name: String,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "restore", about = "Restore information in a backup instance")]
struct RestoreOpts {
    #[structopt(short, long)]
    /// Activate debug mode
    debug: bool,
    #[structopt(short, long)]
    /// The repository to read the backup from
    repository: String,
    #[structopt(short, long)]
    /// The name under which the backup was stored
    name: String,
    #[structopt(short, long)]
    /// The path to restore to
    target: String,
    #[structopt(short, long)]
    /// Filters for the objects to restore.
    ///
    /// This parameter takes a list of regular expressions to filter
    /// the objects to restore. Only objects, whose names match any of the
    /// filter expressions will be restored to the target. If no filter
    /// is given, all objects will be restored.
    include: Option<Vec<String>>,
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
            is_warning: false,
        })?;
    let program_result = match options {
        Opts::Init(opts) => program::initialize_repository(&opts.repository),
        Opts::Create(opts) => create::make_backup(
            &opts.repository,
            &opts.sources,
            &cache_dir,
            &opts.name,
            &merge_exclude(&opts.exlude, &opts.exclude_from)?,
        ),
        Opts::Instances(opts) => instances::instances(&Path::new(&opts.repository)),
        Opts::Show(opts) => show::show(&Path::new(&opts.repository), &opts.name, opts.contents),
        Opts::Restore(opts) => {
            restore::restore_backup(&opts.repository, &opts.target, &opts.include, &opts.name)
        }
    };
    program_result
}

fn merge_exclude(
    exclude: &Option<Vec<String>>,
    exclude_from: &Option<PathBuf>,
) -> backrub::errors::Result<Option<Vec<String>>> {
    let exlusion_entries = if let Some(path) = exclude_from {
        let input_file =
            File::open(path).or_else(|e| error("Could not load exclusion list", Some(e.into())))?;
        let lines = BufReader::new(input_file).lines();
        Some(
            lines
                .collect::<Result<Vec<String>, std::io::Error>>()
                .or_else(|e| error("Could not read exclusion file", Some(e.into())))?,
        )
    } else {
        None
    };
    Ok(Some(
        [&exlusion_entries, exclude]
            .iter()
            .filter(|list| list.is_some())
            .flat_map(|e| (*e).clone())
            .flatten()
            .collect(),
    ))
}
