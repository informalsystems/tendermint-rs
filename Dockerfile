# Tendermint KMS Dockerfile

FROM centos:7.5.1804

# Install/update RPMs
RUN yum update -y && \
    yum groupinstall -y "Development Tools" && \
    yum install -y libusbx-devel openssl-devel sudo && \
    yum clean all && \
    rm -rf /var/cache/yum

# Create "developer" user
RUN useradd developer && \
    echo 'developer ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/developer

# Switch to the "developer" user
USER developer
WORKDIR /home/developer

# Include cargo in the path
ENV PATH "$PATH:/home/developer/.cargo/bin"

# Install rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    rustup update && \
    rustup component add rustfmt-preview && \
    rustup component add clippy-preview && \
    cargo install cargo-audit

# Configure Rust environment variables
ENV RUSTFLAGS "-Ctarget-feature=+aes"
ENV RUST_BACKTRACE full
