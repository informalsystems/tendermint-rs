# Cosmos KMS Dockerfile

FROM centos:7.5.1804

# Install/update RPMs
RUN yum update -y && \
    yum groupinstall -y "Development Tools" && \
    yum install -y centos-release-scl sudo && \
    yum install -y --enablerepo=centos-sclo-rh llvm-toolset-7

# Create "developer" user
RUN useradd developer && \
    echo 'developer ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/developer

# Switch to the "developer" user
USER developer
WORKDIR /home/developer

# Include cargo in the path
ENV PATH "$PATH:/home/developer/.cargo/bin"

# Install rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Rust nightly version to install
ENV RUST_NIGHTLY_VERSION "nightly-2018-07-14"

# Install Rust nightly
RUN rustup install $RUST_NIGHTLY_VERSION

RUN sudo -e "echo $(rustc --print sysroot)/lib >> /etc/ld.so.conf"
RUN sudo ldconfig

# Install rustfmt
RUN rustup component add rustfmt-preview --toolchain $RUST_NIGHTLY_VERSION

# Install clippy
ENV CLIPPY_VERSION "0.0.212"
RUN cargo +$RUST_NIGHTLY_VERSION install clippy --vers $CLIPPY_VERSION

# Set environment variables to enable SCL packages (llvm-toolset-7)
ENV LD_LIBRARY_PATH=/opt/rh/llvm-toolset-7/root/usr/lib64
ENV PATH "/opt/rh/llvm-toolset-7/root/usr/bin:/opt/rh/llvm-toolset-7/root/usr/sbin:$PATH"
ENV PKG_CONFIG_PATH=/opt/rh/llvm-toolset-7/root/usr/lib64/pkgconfig
ENV X_SCLS llvm-toolset-7

# Configure Rust environment variables
ENV RUSTFLAGS "-Ctarget-feature=+aes"
ENV RUST_BACKTRACE full
