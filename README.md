# backrub de-duplicating backup

backrub is a de-duplicating backup program, that stores backups encrypted. This
means, that you get all the benefits of having securely encrypted, full snapshot
backups without exploding storage requirements.

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