# we use a nodejs image so that we don't have to bother installing node
FROM node:lts-bookworm AS builder

RUN apt update
RUN apt install sudo

RUN useradd -m builder && echo "builder:builder" | chpasswd && adduser builder sudo
# Allow builder to run sudo commands without a password
RUN echo "builder ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

USER builder

# Install Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
ENV PATH="/home/builder/.cargo/bin:${PATH}"

COPY . /src

RUN sudo chown -R builder:builder /src

WORKDIR /src

# install all node deps
RUN <<EOF
cd frontend
npm install
EOF

# build rust backend
RUN <<EOF
cargo clean
# build static bin
RUSTFLAGS="-Ctarget-feature=+crt-static" cargo build-app --release --target=x86_64-unknown-linux-gnu
EOF

# Stage 2: Create the final image
FROM alpine:latest

# Copy the built application from the builder stage
COPY --from=builder /src/target/x86_64-unknown-linux-gnu/release/sandbox-ui /usr/local/bin/sandbox-ui

# Install system tools
RUN apk add vim curl python3 git

# Install tools required by kunai-sandbox and deps
RUN apk add openssh tcpdump graphviz

# Install qemu
RUN apk add qemu-system-x86_64 qemu-system-aarch64

# Install python uv
RUN curl -LsSf https://astral.sh/uv/install.sh | env UV_INSTALL_DIR="/usr/local/bin/" sh

RUN adduser -D sandbox-ui

USER sandbox-ui

# Install kunai-sandbox, version is hardcoded to avoid compatibility issues
RUN uv tool install git+https://github.com/kunai-project/sandbox.git@v0.1.5

# Add uv tools path to PATH
ENV PATH=$PATH:/home/sandbox-ui/.local/bin

# Set the entrypoint to your application
ENTRYPOINT ["/usr/local/bin/sandbox-ui"]