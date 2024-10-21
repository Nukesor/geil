# Geil

<p align="center">
    <img src="https://github.com/Nukesor/images/blob/main/geil.gif?raw=true">
</p>

A tool for updating all of your repositories and keeping them clean.

## Features

- Watch folders for new repositories
- Fetch new commits from remote
- Fast-forward only update
- Check git stash size for each repository
- Check for local changes
- Ignore specific repositories
- Execute shell commands after a successful update.

## Add repositories

There are two ways to tell `geil` to update your repositories.

1. Individually add repositories via `geil add $path_to_repository`.
2. Let `geil` watch a whole directory via `geil watch $dir_to_watch`.
   Every time `geil` is started, it will automatically detect new repositories that're up to 5 levels deep in that folder.

## Update your repository

Just call `geil update` to check all repositories.
If you have many repos, you can also specify the thread count via `--threads $count`.

Take a look at the commandline options of each command via the `--help` flag, e.g. `geil update --help`.

## SSH Keychain

If your SSH key is password protected, `geil` needs that key to be in your keychain.
It's not yet supported to ask for the password and use it only in the scope of the current run.

**But** `geil` can check if a list of known keys has been added to `ssh-agent`.
If one of those keys isn't added yet, it will call the command to add it to the keychain for you.

To enable this behavior, just run `geil keys add $path_to_private_keyfile`. \
You can take a look at the registered keys via `geil keys list`.
