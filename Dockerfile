###################################################
# Test harness for remote signer from Tendermint

# Configure the version of Tendermint here against which you want to run
# integration tests
ARG TENDERMINT_VERSION=latest

FROM tendermint/tm-signer-harness:${TENDERMINT_VERSION} AS harness

USER root

RUN mkdir -p /harness

# We need this script to generate configuration for the KMS
COPY tests/support/gen-validator-integration-cfg.sh /harness/

# Generate the base configuration data for the Tendermint validator for use
# during integration testing. This will generate the data, by default, in the
# /tendermint directory.
RUN tendermint init --home=/harness && \
    tm-signer-harness extract_key --tmhome=/harness --output=/harness/signing.key && \
    cd /harness && \
    chmod +x gen-validator-integration-cfg.sh && \
    TMHOME=/harness sh ./gen-validator-integration-cfg.sh

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
COPY --from=harness /harness /harness

# We need the test harness binary
COPY --from=harness /usr/bin/tm-signer-harness /usr/bin/tm-signer-harness

# We need a secret connection key
COPY tests/support/secret_connection.key /harness/

USER root
# Ensure the /harness folder has the right owner
RUN chown -R developer /harness
USER developer

