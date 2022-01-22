# backrub de-duplicating backup

backrub is a de-duplicating backup program, that stores backups encrypted. This
means, that you get all the benefits of having securely encrypted, full snapshot
backups without exploding storage requirements.

## Usage

backrub implements multiple sub-commands for different use cases.

### Initializing a new repository

backrub stores backup instances in a repository, that needs to be initialized before
use. This sets up the repository structure and cryptographic material.

```sh
backrub init <repository>
```

This sets up the path `<repository>` as a backup repository. The command will as
for a master password to derive the cryptographic material from.

_Attention:_ DO NOT loose this master password. The data in a repository will be
completely inaccessible without this password.

### Creating a new backup instance

The `create` command creates a new backup instance in a given repository. A backup
instance is a collection of objects (e.g. files or directories) at a specific point
in time. Each instance is a complete snapshot of the contained objects. Space in
the repository is saved not by creating different increments over multiple instances,
but by reusing unchanged data blocks.

Instances can be from different sources, that do not have to be related to each
other at all (i.e. different directories or even machines).

```sh
backrub create --name <name> --repository <repository> --sources <source1> <source2> ...
```

This creates a backup instance under the name `<name>` from the given sources in
the given repository.

#### Excluding elements from the backup

The `create` command supports excluding objects, whose names match one of a given
set of regular expressions:

```sh
backrub create -n MyBackup -r /my/repository -s /home -e '\.bak$'
```

This create a backup instance from the `/home` directory, but excludes files
and directories ending in `.bak`.

### Restoring data from a backup

The `restore` command restores data from a specific backup instance. 

```sh
backrub restore -r /my/repository -n MyBackup -t /the/restore/path
```

This restores the contents of the `MyBackup` instance in the repository `/my/repository`
to `/the/restore/path`. 

#### Partial restore

Quite often only a partial restore is required to get back certain data (e.g.
after accidentally deleting a file). backrub supports partial restores by allowing
the user to supply an include-filter in the form of one or more regular expressions
to match the backup object name against. Only objects matching at least one filter
will be restored. If no filter is given, backrub restores the complete instance.

```sh
backrub restore -r /my/repository -n MyBackup -t /the/restore/path -i '\.jpg' '\.png'
```

This call will only restore objects ending in `.jpg` or `.png` (i.e. most likely
only images). All other objects in an instance will be ignored.

## Example backup scripts

See [backrub-scripts](https://github.com/DerNamenlose/backrub-scripts) for an example
of backup scripts using backrub to automate backing up data on a Linux system to a
personal NAS.

## FAQ

### Is it ready for prime time yet?

No. Seriously: no. Neither is the repository format fixed for now, nor does it
support backing up all kinds of data you'd normally expect from a backup
program. If you're interested in using the program watch this space for further
news on that topic.

### What does "de-duplicating" mean?/Does backrub support incremental backups?

Instead of implementing some kind of incremental backup mechanism, a
deduplicating backup saves space by reusing data blocks already stored in former
backups.

backrub splits all data into blocks of varying sizes and tracks each block by
its SHA3-256 checksum. If a block is encountered again, it is not stored twice,
but rather just referenced in the backup object meta data. This way, if data
does not change between backup runs, it is stored only once (apart from a litte
overhead for the meta data being stored twice).

This way each backup instance is, in a sense, a full snapshot, without the added
storage overhead.

Other programs employing similar techniques are [Git](https://git-scm.com/)
itself, or (similar to backrub) [borg](https://borgbackup.readthedocs.io/en/stable/).

### What's with the name?

The name is the bastard child of "backup" (the purpose of the program) and
"rust" (the language its implemented in). Also, having a reliable backup feels
just as good, as getting a backrub, so there's that...
