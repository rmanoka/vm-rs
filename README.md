# VM-RS

Utilities to help with VMs and S3 storage.  Install with git and cargo:

```sh
cargo install --git <git path>
```


# Supported Workflows

## MFA Authentication

Setup device ARN in `~/.config/vm-rs/config.toml`:

```toml
device = "<arn_string>"
```

Example usage:

```sh
vm-rs mfa --check || {
    echo -n "Enter OTP token: "
    read token
    vm-rs mfa "$token"
}
```

The session token is maintained under a `mfa` profile in `~/.aws/credentials`.

## Sync

Sync the current directory (recursively) to S3 or a `rsync`-able remote
directory.  The remote URL is determined by the current directory, and a _prefix
mapping_ config file found in the current directory or any parent directory.

Create a file `.s3-prefix` or `.vm-prefix` that maps a given directory to it's
corresponding remote URL / path.  Descendents of this directory will
automatically be mapped appropriately based on suffix from the directory
containing the prefix mapping file.

Options support syncing to S3 (`-3`), sync only git-ls-files (`-g`), and many
more.

```bash
# prints the rsync command that will be run
vm-rs sync --print

# sync output of `git ls-files` to remote
vm-rs sync -g
```

## VM management

Use `vm-rs vm` to manage VMs.  This is primarily a wrapper around `aws ec2`, but
also supports setting up SSH config (under alias `Host vmrs`).

```bash
vm-rs vm --help
```

