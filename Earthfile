VERSION 0.7
FROM rust:1-bullseye

prepare:
    FROM rust:1
    WORKDIR /code
    RUN cargo install cargo-chef
    RUN apt-get --yes update && apt-get --yes install cmake musl-tools gcc-aarch64-linux-gnu protobuf-compiler
    RUN rustup target add x86_64-unknown-linux-musl
    RUN rustup target add aarch64-unknown-linux-musl

    ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=/usr/bin/aarch64-linux-gnu-gcc
    ENV CC_aarch64_unknown_linux_musl=/usr/bin/aarch64-linux-gnu-gcc

    SAVE IMAGE --push ghcr.io/mortenlj/suffiks-ingress/cache:prepare

chef-planner:
    FROM +prepare
    COPY --dir src proto build.rs Cargo.lock Cargo.toml .
    RUN cargo chef prepare --recipe-path recipe.json
    SAVE ARTIFACT recipe.json

chef-cook:
    FROM +prepare
    COPY +chef-planner/recipe.json recipe.json
    ARG target
    RUN cargo chef cook --recipe-path recipe.json --release --target ${target}
    SAVE IMAGE --push ghcr.io/mortenlj/suffiks-ingress/cache:chef-cook-${target}

build:
    FROM +chef-cook

    COPY --dir src proto build.rs Cargo.lock Cargo.toml .
    ARG target
    RUN cargo build --release --target ${target}

    SAVE ARTIFACT target/${target}/release/suffiks-ingress suffiks-ingress
    SAVE IMAGE --push ghcr.io/mortenlj/suffiks-ingress/cache:build-${target}

docker:
    FROM cgr.dev/chainguard/static:latest

    WORKDIR /bin
    ARG target=x86_64-unknown-linux-musl
    COPY --platform=linux/amd64 (+build/suffiks-ingress --target=$target) suffiks-ingress

    CMD ["/bin/suffiks-ingress"]

    # builtins must be declared
    ARG EARTHLY_GIT_SHORT_HASH

    ARG REGISTRY=ghcr.io/mortenlj/suffiks-ingress
    ARG image=${REGISTRY}/suffiks-ingress
    ARG VERSION=$EARTHLY_GIT_SHORT_HASH
    SAVE IMAGE --push ${image}:${VERSION} ${image}:latest

manifests:
    FROM dinutac/jinja2docker:latest
    WORKDIR /manifests
    COPY deploy/* /templates
    ARG REGISTRY=mortenlj/suffiks-ingress
    ARG VERSION=$EARTHLY_GIT_SHORT_HASH
    ARG image=${REGISTRY}/suffiks-ingress
    RUN --entrypoint -- /templates/application.yaml.j2 /templates/variables.toml --format=toml > ./deploy.yaml
    # RUN cat /templates/*.yaml >> ./deploy.yaml
    SAVE ARTIFACT ./deploy.yaml AS LOCAL deploy.yaml

deploy:
    BUILD --platform=linux/amd64 +prepare
    BUILD --platform=linux/arm64 +docker --target=aarch64-unknown-linux-musl
    BUILD --platform=linux/amd64 +docker --target=x86_64-unknown-linux-musl
    BUILD +manifests
