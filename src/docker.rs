use anyhow;
use log::{debug, error, info, warn};
use serde::Serialize;
use std::path::Path;
use std::process::Command;
use tinytemplate::TinyTemplate;

// TODO: don't allow injection!
static DOCKERFILE_UBUNTU_TEMPLATE: &'static str = "\
FROM {base_image} AS builder

RUN apt update
RUN apt install -yq wget lld
RUN wget https://sh.rustup.rs -O install-rustup.sh
RUN sh install-rustup.sh -y
ENV PATH=$PATH:/root/.cargo/bin
RUN rustup toolchain install stable
# TODO: Fix the empty case
RUN apt install -yq {build_deps}
RUN apt install -yq clang

RUN mkdir /work /artifacts
WORKDIR /work
COPY target /work/target
COPY Cargo.* /work/
RUN --mount=type=bind,source=src,target=src \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build && cp /work/target/debug/{artifact} /artifacts/

FROM {base_image} AS runner
RUN apt update
# TODO: Fix the empty case
RUN apt install -yq {runtime_deps}
COPY --from=builder /artifacts/{artifact} /usr/bin/{artifact}

ENTRYPOINT [\"/usr/bin/{artifact}\"]
";

// TODO: Need sanitization (value check) before embedding into the
// template to prevent malicious injection.
#[derive(Debug, Serialize)]
pub struct ContainerBuildConfig {
    pub target_image: String,
    pub base_image: String,
    pub build_deps: String,
    pub runtime_deps: String,
    pub artifact: String,
}

#[derive(Debug, PartialEq)]
pub enum CacheMode {
    Local,
    Gha,
}

/// TODO: optimize cache
pub fn build(config: &ContainerBuildConfig, mode: CacheMode) -> anyhow::Result<()> {
    let mut filename = std::env::current_dir()?;
    filename.push("target");
    std::fs::create_dir_all(&filename)?;
    // TODO: do some suffix work to make it deterministic
    filename.push("tmp-dockerfile");

    // Render and save the Dockerfile
    dockerfile(config, filename.as_path())?;

    let mut binding = Command::new("docker");
    let mut b = binding
        .arg("buildx")
        .arg("build")
        .arg("-t")
        .arg(config.target_image.to_string())
        .arg("-f")
        .arg(filename.as_path());

    if mode == CacheMode::Gha {
        b = b
            .arg("--cache-from")
            .arg("type=gha,id=deadbeef")
            .arg("--cache-to")
            .arg("type=gha,mode=max,id=deadbeef")
            .arg("--output=type=docker")
            .arg("--iidfile=target/build-iidfile.txt")
            .arg("--metadata-file=target/build-metadata.json");
        // or .arg("--output=type=registry")
        warn!("You look you need type=gha, but GHA cache is somehow not working.");
    }

    let cmd = b.arg(".");

    info!("{:?}", cmd.get_args().collect::<Vec<_>>());
    let mut p = cmd.spawn()?;
    let status = p.wait()?;
    if status.success() {
        Ok(())
    } else {
        error!("Exit fail: {:?}", status);
        anyhow::bail!("fail");
    }
}

pub fn push(target_image: &String) -> anyhow::Result<()> {
    let mut b = Command::new("docker");
    let cmd = b.arg("push").arg(target_image);
    info!("{:?}", cmd.get_args().collect::<Vec<_>>());
    let mut p = cmd.spawn()?;
    let status = p.wait()?;
    if status.success() {
        Ok(())
    } else {
        error!("Exit fail: {:?}", status);
        anyhow::bail!("fail");
    }
}

pub fn dockerfile(config: &ContainerBuildConfig, dockerfile: &Path) -> anyhow::Result<()> {
    let mut tt = TinyTemplate::new();
    tt.add_template("dockerfile", DOCKERFILE_UBUNTU_TEMPLATE)?;
    let rendered = tt.render("dockerfile", &config)?;
    debug!("{}", rendered);
    std::fs::write(dockerfile, rendered)?;
    debug!("{} saved", dockerfile.display());
    Ok(())
}
