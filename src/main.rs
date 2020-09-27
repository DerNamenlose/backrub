use crate::backupobject::BackupObjectWriter;
use fssource::FsBlockSource;
use fssource::FsSource;
use repository::{FsRepository, Repository};
use structopt::StructOpt;

mod backupobject;
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
    let repo: FsRepository = Repository::new(&opts.repository);
    let source: FsSource = FsSource::new(&opts.path);

    repo.initialize();
    for file in source.objects() {
        let source_name = file.path().to_str();
        if source_name.is_some() {
            let blocks_result = source.open_entry(&source_name.unwrap());
            let object_result = repo.start_object(&source_name.unwrap());
            match (blocks_result, object_result) {
                (Ok(blocks), Ok(mut object)) => {
                    let copy_result = copy_blocks(blocks, object.as_mut());
                    match copy_result {
                        Ok(()) => {
                            println!("Adding object descriptor to repository");
                            let finish_result = object.finish();
                            match finish_result {
                                Ok(id) => {
                                    println!("New object: {}", id);
                                }
                                Err(message) => println!(
                                    "Could not finish object {}. Reason: {}",
                                    source_name.unwrap(),
                                    message
                                ),
                            }
                        }
                        Err(message) => println!(
                            "Could not copy blocks for {}. Reason: {}",
                            source_name.unwrap(),
                            message
                        ),
                    }
                }
                (_, _) => println!("Could not copy source blocks into target object"),
            }
        }
    }
}

fn copy_blocks(
    blocks: FsBlockSource,
    object: &mut dyn BackupObjectWriter,
) -> Result<(), &'static str> {
    for block in blocks {
        object.add_block(&block)?;
    }
    println!("Finished copying blocks");
    Ok(())
}
