# Tendermint KMS Dockerfile

FROM centos:7

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
ENV RUSTFLAGS "-Ctarget-feature=+aes"
ENV RUST_BACKTRACE full
