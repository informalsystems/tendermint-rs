###################################################
# Test harness for remote signer from Tendermint

# Configure the version of Tendermint here against which you want to run
# integration tests
ARG TENDERMINT_VERSION=latest

FROM tendermint/remote_val_harness:${TENDERMINT_VERSION} AS harness

USER root

RUN mkdir -p /remote_val_harness

# We need this script to generate configuration for the KMS
COPY tests/support/gen-validator-integration-cfg.sh /remote_val_harness/

# Generate the base configuration data for the Tendermint validator for use
# during integration testing. This will generate the data, by default, in the
# /tendermint directory.
RUN tendermint init --home=/remote_val_harness && \
    remote_val_harness extract_key --tmhome=/remote_val_harness --output=/remote_val_harness/signing.key && \
    cd /remote_val_harness && \
    chmod +x gen-validator-integration-cfg.sh && \
    TMHOME=/remote_val_harness sh ./gen-validator-integration-cfg.sh

###################################################
# Tendermint KMS Dockerfile

FROM centos:7 AS build

# Install/update RPMs
RUN yum update -y && \
    yum groupinstall -y "Development Tools" && \
    yum install -y \
    centos-release-scl \
    cmake \
    epel-release \
    libudev-devel \
    libusbx-devel \
    openssl-devel \
    sudo && \
    yum install -y --enablerepo=epel libsodium-devel && \
    yum install -y --enablerepo=centos-sclo-rh llvm-toolset-7 && \
    yum clean all && \
    rm -rf /var/cache/yum

# Set environment variables to enable SCL packages (llvm-toolset-7)
ENV LD_LIBRARY_PATH=/opt/rh/llvm-toolset-7/root/usr/lib64
ENV PATH "/opt/rh/llvm-toolset-7/root/usr/bin:/opt/rh/llvm-toolset-7/root/usr/sbin:$PATH"
ENV PKG_CONFIG_PATH=/opt/rh/llvm-toolset-7/root/usr/lib64/pkgconfig
ENV X_SCLS llvm-toolset-7

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
    rustup component add rustfmt && \
    rustup component add clippy && \
    cargo install cargo-audit

# Configure Rust environment variables
ENV RUSTFLAGS "-Ctarget-feature=+aes,+ssse3"
ENV RUST_BACKTRACE full

###################################################
# Remote validator integration testing

# We need the generated harness and Tendermint configuration
COPY --from=harness /remote_val_harness /remote_val_harness

# We need the test harness binary
COPY --from=harness /usr/bin/remote_val_harness /usr/bin/remote_val_harness

# We need a secret connection key
COPY tests/support/secret_connection.key /remote_val_harness/

USER root
# Ensure the /remote_val_harness folder has the right owner
RUN chown -R developer /remote_val_harness
USER developer
