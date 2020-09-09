# Safe API/CLI support for symlinks

| [MaidSafe website](https://maidsafe.net) | [Safe Dev Forum](https://forum.safedev.org) | [Safe Network Forum](https://safenetforum.org) |
|:----------------------------------------:|:-------------------------------------------:|:----------------------------------------------:|

# For users of safe-cli

## What is a symlink?

Symlinks have long been a feature of Unix filesystems, and are now also
available on Windows ([with some restrictions](#enabling-symlinks-on-windows)).
A symlink is a special node in a filesystem that is neither a file or a
directory, but rather points to a file or directory or another symlink, called
the target. Multiple symlinks may reference a single target. Symlinks are useful
for creating shortcuts to any path in the filesystem.

Symlinks may be either absolute or relative. A symlink is called absolute if the
target path begins with the root of the filesystem, ie `/` in Unix or `c:\` in
Windows. A symlink is called relative if the target path is just a name, or begins
with `./` or `../`, in which case the target location is relative to the
symlink's path.

For more information, see the [wikipedia
entry](https://en.wikipedia.org/wiki/Symbolic_link).

## Safe Network support for symlinks

The Safe Network File API permits storing, retrieving, and resolving symlinks
within a FileContainer. This is necessary for faithful storage and retrieval of
a local directory tree that contains symlinks.

Absolute symlinks may be stored and retrieved, but will not resolve. In other
words, the absolute symlink may work on your local filesystem, but will be
*broken* on the safe network. Upon retrieval to your local filesystem, it will
work once again, provided the target still exists.

Relative symlinks may be stored and retrieved, and will resolve only if the
target is within the FileContainer. In other words, if you upload a directory
tree with relative symlinks that reference targets within the uploaded
directory, then those links will work in the Safe Network. But if the link
targets a path outside the uploaded directory, then it will not work, because
the target has not been uploaded.

For public data it is considered good etiquette to only include relative
symlinks with targets inside the uploaded directory.

Finally, symlinks may optionally be [disabled](#disabling-symlinks) when
uploading content to the Safe Network.

### Can I make a symlink to another file container on the network?

No. Symlinks in Safe Network are simply a mechanism for supporting the symlinks
that may already exist in your local filesystem.

Links from one FileContainer to another may happen at some point, but introduce
considerable complexity, so would need to be carefully designed. As of this
writing there is no plan for such.

## Storing symlinks

Symlinks are stored in the FileContainer as metadata when using the `safe files
put` command. This is also how directories are stored. Typically, symlinks would
be stored when recursively uploading a directory tree.

### Disabling symlinks

All `safe-cli` commands for uploading files now have a flag `--follow-links`.
When this flag is present, the symlink target file or directory will be
uploaded instead of the symlink.  This occurs even if the target is outside
of the original upload directory, so it can potentially result in larger
uploads than expected.

## Retrieving symlinks

Symlinks are retrieved just like any other path using the `safe files get`
command. Typically, symlinks would be retrieved when recursively downloading a
directory tree.

## Viewing symlinks

Symlinks in a FileContainer can easily be viewed with `files tree`, `files ls`,
or `cat`. For example, here is the output of `files tree`:

```
safe://hnyynywjyjgxjbgzfxgsb31zexfcrafdkzu3otj1ixuyr8gaaof3tdgrqqbnc
├── absolute_links.txt
├── broken_rel_link.txt -> non-existing-target
├── dir_link -> sub
├── dir_link_deep -> sub/deep
├── dir_link_link -> dir_link
├── dir_outside -> ../
├── file_link -> realfile.txt
├── file_link_link -> file_link
├── file_outside -> ../file_outside
├── realfile.txt
├── sub
│   ├── deep
│   ├── infinite_loop -> infinite_loop
│   ├── parent_dir -> ..
│   ├── parent_dir_file_link.txt -> ../realfile.txt
│   ├── readme.md
│   ├── sibling_dir -> ../sub2
│   ├── sibling_dir_file.md -> ../sub2/hello.md
│   └── sibling_dir_trailing_slash -> ../sub2/
└── sub2
    ├── hello.md
    └── sub2 -> ../sub2
```

## Referencing symlinks in a URL

If a SafeUrl contains a path component, all symlinks in the path will be
resolved to their targets and a final "real" path to a file or directory in the
FileContainer is generated. If that path exists, then the resource can be
returned, else an error will be generated.

## Symlinks and Windows

Windows since Vista supports native symlinks, however they are disabled by
default. Writing symlinks to disk requires certain permissions (depends on the
exact OS version), so `safe files get` may skip the symlink and issue a warning
in this case. 

Symlink support is enabled by choosing [run as
administrator](https://www.howtogeek.com/howto/16226/complete-guide-to-symbolic-links-symlinks-on-windows-or-linux/)
when starting a command terminal, or in Windows 10+ one may [enable Developer
Mode](https://www.ghacks.net/2016/12/04/windows-10-creators-update-symlinks-without-elevation).

# Technical Details

## How symlinks are stored

Symlinks are stored as FileItem metadata in a FileContainer. The following
fields are relevant for symlink processing:

|field                   | description |
|--                      | --          |
|**type**                | identifies FileItem as symlink when the value is `inode/symlink`|
|**symlink_target**      | path to the symlink's target.  may be relative or absolute |
|**symlink_target_type** | type of the target.  can be: `dir`, `file`, or `unknown` |


The JSON for a FileContainer with a single symlink looks like:

```
[
  "safe://hnyynywxqqg7p1ftge8zzk9ujr6x1gnfpcmymrqf5ymet39m45rhpt8jkcbnc",
  {
    "/a_symlink": {
      "created": "2020-06-16T16:20:32Z",
      "mode_bits": "41471",
      "modified": "2020-06-16T16:20:32Z",
      "o_created": "2020-06-16T16:20:15Z",
      "o_modified": "2020-06-16T16:20:15Z",
      "readonly": "false",
      "size": "0",
      "symlink_target": "somefile.txt",
      "symlink_target_type": "file",
      "type": "inode/symlink"
    }
  }
]
```

## API Modifications

A parameter `follow_links` was added to the following public APIs:

* Safe::files_container_create()
* Safe::files_container_sync()
* Safe::files_container_add()
* Safe::files_map_sync()

When follow_links is true, these functions resolve the symlink before uploading,
so it is stored as a file or directory. When false, no path resolution is
performed and it is stored as a symlink.

A trait RealPath was added to FilesMap::realpath() for resolving paths
containing ./ ../ and relative symlinks. This works like the standard realpath()
function on a Unix system, but understands structure/metadata in a FilesMap.

Safe::fetch() and Safe::inspect() APIs were modified to resolve URL paths via
FilesMap::realpath()

SafeUrl (XorUrlEncoder) now uses a different URL parser to obtain the raw path,
because rust-url normalizes '../' away.

## Path Resolution

If a SafeUrl contains a path component, the path will be resolved by
FilesMap::realpath(), which is a Safe Network adaptation of the standard Unix
realpath() function. This function checks each component of the path from the
beginning, and if it finds a relative symlink with a valid target, then it
substitutes the target path and continues processing path components. Each
target path may itself contain symlinks to be further resolved before continuing,
If links descend beyond 16 levels, an error is returned.

One detail to highlight is that `../` in a SafeUrl path is resolved after any
preceding symlinks. So it works as would be expected in a local filesystem. Here
is an example:

```
$ safe files tree safe://hnyynyi9npmuhnsyfk5a31umcfh4macnwa5f4kq1wb7bhgpypnug6qgaqhbnc
safe://hnyynyi9npmuhnsyfk5a31umcfh4macnwa5f4kq1wb7bhgpypnug6qgaqhbnc
├── level2
│   ├── hello.txt
│   └── level3
└── level3 -> level2/level3

$ safe cat safe://hnyynyi9npmuhnsyfk5a31umcfh4macnwa5f4kq1wb7bhgpypnug6qgaqhbnc/level3/../hello.txt hello!
```

In the above example, if the `../` were resolved from the path in the URL alone
as per the WHATWG URL spec and the rust-url implementation, it would result in the final
path `/hello.txt`, which does not exist in the FileContainer, so the `safe cat`
command would fail. Instead, the resolver first resolves the `/level3` symlink
to `/level2/level3`, then resolves `../` to `/level2` and finally appends
`/hello.txt` giving the correct path `/level2/hello.txt`.

## Integration Testing

Test cases for symlinks have been added for commands: `cat`, `files ls`, `files tree`, and `files get`.

These tests can be divided into relative symlink tests and absolute symlink tests.

Relative symlink tests can be checked into git and should perform the same on
different machines. Absolute symlink tests would often give different results on
each machine because the paths outside the testing repository directory will be
different.

For this reason, it was decided to commit a test directory tree for the relative symlink
tests to operate on, while absolute symlink tests generate a tree dynamically
for each test within the system temp directory.

The relative symlinks test directory resides at path sn_api/test_symlinks.

See [Integration Testing on Windows](#integration-testing-on-windows)

## Windows-specific behavior

### Symlink Creation APIs

Rust has platform specific APIs for creating symlinks.  On Unix, a single API
`std::os::unix::fs::symlink(target, link)` creates a symlink regardless of the
target.  On Windows, two APIs exist:

```
std::os::windows::fs::symlink_file(target, link)
std::os::windows::fs::symlink_dir(target, link)
```

This makes it necessary to know the type of the target, even if the target no
longer exists. This is one reason that we record the target type in the
`FileItem` `symlink_target_type` field.

### Permissions

Writing symlinks to disk requires administrator permissions, so `safe files get`
will skip the symlink and log/print a warning in this case.

Git also supports symlinks and faces the same issue. When Git on windows fails
to create a symlink, it falls back to creating a text file containing the target
path instead. If the file gets changed/committed locally, git can translate that
back into a symlink in the repository because it has metadata in the .git
storage. safe-cli does not have this local metadata, so a download -> upload
cycle would cause the original symlink to be uploaded (PUT) as a file instead of
a symlink.  This alteration of the data seems undesirable, so it was judged better
to skip the symlink entirely and warn the user so they can take corrective action.

Hopefully Microsoft removes this restriction in a future release of Windows.

### Integration Testing on Windows

On Windows machines, it is important that the git repository be cloned within a
shell (command prompt) that is opened with the "Run as Administrator" option, or
that the machine is running Windows 10+ and has "Developer Mode" enabled. If
these requirements are not met, (a) git will be unable to create the relative
symlink directory correctly causing tests that read symlinks to fail and (b)
`safe files get` will be unable to write symlinks causing those tests to fail.



