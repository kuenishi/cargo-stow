# Containers in Rust: no more Dockerfile

In the era of containers and microservices, we often build a
single-feature container image that just runs one executable as an
entrypoint. We just want to stow a binary executable onto a container
layer. We don't need complex assets, pre-built libraries with complex
build scripts and configs, which just requires a single binary
`/usr/bin/your-executable` written in Rust.  We don't want to write
Dockerfile just for it, copying a plain Dockerfile just copied by some
random other Rust binary crate.

Another burden is that we have to take the difference between the
build environment and the runtime image into consideration. We don't
need headers, protoc command, but we just need dynamic libraries and
some assets. Even configuration files will always be injected by a
container orchestrator. We often use `rust` image for build and
`debian-slim` for runtime. But they do often have _difference_ in
library versions and ABIs and other version dependencies. So, the
build system and the runtime system should stem from the same base
container image. The same-base-image rule should be forced, because we
usually don't mind or get noticed when the gap happens..

Rather, we just add a single section in `Cargo.toml` manifest named
`[package.metadata.container]` that describes container image name
and, by a single subcommand `cargo stow build` , we obtaing a cleanly
built container image. That won't bother us any more as long as this
crate is correctly designed.

Some notes:
- This project is not related to GNU Stow. When I hit on the name
  cargo-stow, I didn't even know it.
- `cargo-chef` project has some part common with this project,
  especitally in the domain of building container images in Rust. For
  example, cache optimization and such.
- In some day, I want to make this tool purly hosted with Rust. But to
  prove my idea of simplifying build process, I dared to use Docker as
  it is the most common tool.

## Install and Prerequisites

A working `docker` command with is required. To install the `cargo-stow`, run
```sh
$ docker info
...
$ docker buildx version
...
$ cargo install cargo-stow
...
```

## Subcommands

- `cargo stow build`: build an OCI-compatible container
- `cargo stow push`: push the built container
- `cargo stow run`: run the Rust program inside a temporarily built container (TODO)
- `cargo stow dockerfile`: Generate the Dockerfile

## Features:
- `docker`: Use Docker for build and push backend (Default, needs `docker` command)
- `youki`: Use Youki for build backend, and push the artifact natively (TODO)


# License

See `LICENSE` file. By contributing, you agree that your contributions will be licensed under the same license as in `LICENSE` file.
