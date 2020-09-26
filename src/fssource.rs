use std::sync::mpsc::sync_channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::thread;
use walkdir::DirEntry;
use walkdir::WalkDir;

pub struct FsSource {
    pub output: Receiver<DirEntry>,
    thread: thread::JoinHandle<()>,
}

fn source_main(path: String, sender: SyncSender<DirEntry>) -> () {
    for file in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| f.file_type().is_file())
    {
        let result = sender.send(file);
        if result.is_err() {
            println!("Could not write to output");
            break;
        }
    }
    drop(sender);
}

pub fn start(path: String) -> FsSource {
    println!("Starting FsSource thread for path {}", path);
    let (sender, receiver) = sync_channel(100);
    let thread = thread::spawn(|| source_main(path, sender));
    FsSource {
        output: receiver,
        thread: thread,
    }
}

pub fn stop(source: FsSource) -> () {
    let result = source.thread.join();
    match result {
        Ok(_) => {}
        Err(_) => println!("Could not stop repository thread"),
    }
}
