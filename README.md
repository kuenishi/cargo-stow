# Containers in Rust: no more Dockerfile

- `cargo stow build`: build a container
- `cargo stow push`: push the built container
- `cargo stow run`: run the Rust program inside a temporarily built container (TODO)
- `cargo stow dockerfile`: Generate the Dockerfile

Features:
- `docker`: Use Docker for build and push backend (Default, needs `docker` command)
- `youki`: Use Youki for build backend, and push the artifact natively (TODO)


# License

See `LICENSE` file. By contributing, you agree that your contributions will be licensed under the same license as in `LICENSE` file.
