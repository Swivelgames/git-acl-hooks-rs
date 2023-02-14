# git-acl-hook

A simple git-hook written in Rust that provides a rudimentary ACL functionality.

Inspired by [seryl's git-acl-hooks](https://github.com/seryl/git-acl-hooks).

## Prerequisites / Assumptions

1. You have the ability to build Rust applications
2. You have access to the `.git/hooks` directory of your remote repository

## Setup

1. Build the project using `cargo`
2. Define, commit, and push up an `acl` file
3. Copy `git-acl-hook` into your remote `.git/hooks/` and name it `update`

## Usage and Syntax

The `acl` file syntax is comprised of sections made up of the following:

```acl
/regular-expression/
access = ACCESS_TYPE
users = USER1 USER2
```

The file format allows for commands for lines that begin with `#`.

- Each section starts with a regular expression that supports `u` and `i` flags.
- `access` can have one of two possible values: `read-only` or `read-write`.
- `users` is a whitespace delineated list of case-sensitive POSIX user names.
  - `users` also accepts a wildcard for `*` to match all users

The default behavior is to allow Read-Write access to all files, and the rulesets
function as follows:

If a changed file matches a Read-Write section's regular expression:

- The file is restricted to the users listed in the section.

If a changed file matches a Read-Only section's regular expression:

- The file is writable by any user not listed in the section.

To change this behavior, you can set `default` at the top of the acl file to `read-only`.

Example:

```acl
default = read-only

/^some-file/
access = read-write
users = user1
```

In this case, all files will be Read-Only unless explicitly given `read-write`.

## Custom ACL file

A custom ACL file can be used by setting the `GIT_ACL_HOOK_FILE`
environment variable.

## Limiting changes to the `acl` file

It's important to make sure that changes to the `acl` file is limited. In the
event the `acl` file isn't regulated, a warning will be outputted stating that
the `acl` file isn't regulated.

## Complete ACL File example

```acl
# Everything is read-write by default
# Uncomment the below to by read-only
# by default:
#default = read-only

/^acl$/
access = read-write
users = adminuser adminuser2

# This sets up read-write access to a folder
/^lib/
access = read-write
users = adminuser
```
