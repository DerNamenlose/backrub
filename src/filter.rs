use walkdir::DirEntry;

/**
 * The type definition for a filter function filtering objects from the backup
 */
pub type FilterFn = dyn Fn(&DirEntry) -> bool; // TODO: this is too tightly bound to the objects in question being from a filesystem
