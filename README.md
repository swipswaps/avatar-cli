# Avatar CLI

[![Pipeline Status]][Dev Commits] [![Latest Version]][crates.io] ![License]

## Introduction

Avatar-CLI is a statically-liked single-binary command line tool meant to ease
the development forkflow in many ways:
  - Making possible version-pinning for any kind of tool used in any kind of
    project. No need for complex setups or ultra-specific tools like
    [`nvm`](https://github.com/nvm-sh/nvm),
    [`nodenev`](https://ekalinin.github.io/nodeenv/),
    [`pyenv`](https://github.com/pyenv/pyenv),
    [`rbenv`](https://github.com/rbenv/rbenv),
    [`goenv`](https://github.com/syndbg/goenv),
    [`asdf-vm`](https://asdf-vm.com), ... I guess you already have seen the
    pattern.
  - Making possible for new contributors to be productive from the very first
    minute, reducing the bootstrap/setup time to almost zero. Only `git` and
    `docker` are required.

## How to install Avatar-CLI

### Downloading pre-compiled binaries

You can get our pre-compiled binaries in the
[Releases](https://gitlab.com/avatar-cli/avatar-cli/-/releases) section. For now
we only can provide Linux binaries, but in the future we'll also provide
binaries for Macos.

### Via Cargo

If you want to use Avatar-CLi in Macos, or you don't mind waiting a little bit
more for its compilation, you can use `cargo` to install it:

```bash
cargo install avatar-cli
```

If you don't have `cargo` in your system, you can obtain it via
[rustup](https://rustup.rs/).

## How to use Avatar-CLI

In the command line:
```bash
# 1. Enter into your project directory
cd /your/project/path

# 2. Initialize Avatar-CLI for this project, this will create a new config file.
#    You only have to do this one single time per project.
avatar init

# 3. Edit the generated configuration file, without modifying its
#    `internalProjectId` property. You can see an example in the next code
#    block of this README.md file.

# 4. Now you can enter into an Avatar-CLI subshell and use all the configured
#    tools. If, for example, you configured a specific version of NodeJS, then
#    it will be available inside that subshell.
avatar shell

# Now you can use the software you specified in the configuration, even if it
# was not installed in your system.
```

Configuration example:
```yaml
---
# This is of no use... for now, but it must be there
version: '0.15.0'

# Keep the value generated by `avatar init`, don't share it across projects.
# This value is used (among other things) to keep track of managed volumes and
# containers
projectInternalId: v2ZmtbkGuVdvGwVE

# In this section we declare the OCI images that we'll use in our project
images:
  # Image name
  node:
    # Image tag
    14-buster:
      # The runConfig block allows us to tweak our containers, to improve their
      # integration with our development environment.
      runConfig:
        # You can specified hardcoded environment variables for your containers,
        # in this example we use this to configure NPM's cache
        env:
          npm_config_cache: /caches/.npm

        # You can pass host environment variables to your containers
        envFromHost:
          - NPM_TOKEN

        # You can specify which container paths have to be mounted as volumes,
        # this is specially useful for package managers' caches
        volumes:
          /caches/.npm: {} # In most cases, we won't need to configure volumes
          /another/example:
            # Volume names are usually autogenerated, use this configuration
            # option only in case you want to share volumes accross projects
            name: exotic_volume

            # By default, the scope of a volume is "Project", allowed values are
            # "Project", "OCIImage" and "Binary".
            # This setting defines how volumes are shared between containers,
            # and has no effect if a custom volume name has been set.
            scope: Project

        # In most cases, bindings won't be necessary, and it's advisable to
        # avoid them as they difficult to share development environments with
        # other people. But, if you really need to map a container path to your
        # host filesystem, they will allow you to do so.
        bindings:
          /container/path: /host/path

      # For each image, we can declare which binaries we want to expose to our
      # project.
      binaries:
        node:
          path: node # The path can also be an absolute path

          # Although we don't do it in this example, each binary can have its
          # own `runConfig` block, and its values will override the ones defined
          # at the image tag level

        # Usually we can skip configuring the binary, we just have to list it
        npm: {}
        npx: {}
        yarn: {}

  # Image name
  rust:
    # Image tag
    1.45-stretch:
      binaries:
        cargo:
          path: cargo
```

## Using Avatar-CLI in CI/CD pipelines

If you want to use Avatar-CLI in your own CI/CD pipelines, you can rely on the
generated OCI images. We provide the Avatar-CLI images through three different
registries:

- **[Gitlab CI Registry](https://gitlab.com/avatar-cli/avatar-cli/container_registry)**:
  `registry.gitlab.com/avatar-cli/avatar-cli:[ major[.minor[.patch]] | latest ]`
- **[Github Registry](https://github.com/avatar-cli/avatar-cli/packages?package_type=Docker)**:
  `docker.pkg.github.com/avatar-cli/avatar-cli/avatar-cli:[ major[.minor[.patch]] | latest ]`
- **[Docker Hub](https://hub.docker.com/r/avatarcli/avatar-cli)**:
  `avatarcli/avatar-cli:[ major[.minor[.patch]] | latest ]`

## Using Avatar-CLI inside scripts

Given that creating subshells inside scripts may be too cumbersome, you can also
"source" the output of the `avatar export-env` command.

In Bash scripts, you could do something like this:
```bash
#!/bin/bash

source <(avatar export-env)
```

If you want full compatibility with POSIX shell, then you have to first create a
file and then source it:
```bash
#!/bin/sh

avatar export-env > /your/temporary/file
. /your/temporary/file
```

## Troubleshooting

### Interactive Git Hooks using tools managed by Avatar-CLI

Git hooks are non-interactive by default, if you want to transform them into
interactive programs, the program must connect its standard input to a
pseud-tty device (usually `/dev/tty`) by itself.

If such git hooks use tools managed by Avatar, you must ensure that the TTY
attachment is performed before any managed container is spawned, because once
the container has been already connected and started, there's no sensible
way to attach a terminal to it for Avatar-CLI.

One example for the mentioned problem is the combination of tools such as
NodeJS, Husky and Commitizen. Husky will start a shell script for each git hook,
and pass the control of execution to `npm` after some checks.

In the specific case of Avatar-CLI, we solved this problem by creating a patch
for Husky ([PR #747](https://github.com/typicode/husky/pull/747)), and [forking
the project](https://www.npmjs.com/package/@coderspirit/husky-fork) while the
Husky project mantainers decide whether to accept this PR or not.

### Customized `$PATH` environment variable

In case you have a customized $PATH environment variable via configuration
scripts like `~/.bashrc`, you should wrap that redefinition with a conditional
statement to avoid breaking how Avatar-CLI works.

For example, something like this:
```bash
export PATH="/custom/extra/bin/path:${PATH}";
```

Should be converted into this:
```bash
if [ -z "${AVATAR_CLI_SESSION_TOKEN}" ]; then
  export PATH="/custom/extra/bin/path:${PATH}";
fi
```

Don't worry, the paths you declared are still available, but Avatar-CLI must be
sure that its own managed paths are the first ones to be evaluated.

## How to contribute

Read the [contributing guidelines](CONTRIBUTING.md).

## License

Avatar-CLI is licensed under the [GPL 3.0 license](LICENSE).

[crates.io]: https://crates.io/crates/avatar-cli
[Dev Commits]: https://gitlab.com/avatar-cli/avatar-cli/-/commits/dev
[License]: https://img.shields.io/crates/l/avatar-cli.svg
[Latest Version]: https://img.shields.io/crates/v/avatar-cli.svg
[Pipeline Status]: https://gitlab.com/avatar-cli/avatar-cli/badges/dev/pipeline.svg
