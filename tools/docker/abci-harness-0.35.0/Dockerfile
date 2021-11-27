FROM alpine:3.15
LABEL maintainer="hello@informal.systems"

ARG TM_VERSION=0.35.0
ARG TM_ARCHIVE_HASH=c70dc4538991183905c1eef17263b713666675a995d154a75a52cf0022338724
ARG GLIBC_VERSION=2.34-r0
ENV TM_HOME=/tendermint

#GLIBC for Alpine from: https://github.com/sgerrand/alpine-pkg-glibc
RUN wget https://alpine-pkgs.sgerrand.com/sgerrand.rsa.pub \
         -O /etc/apk/keys/sgerrand.rsa.pub && \
    wget https://github.com/sgerrand/alpine-pkg-glibc/releases/download/${GLIBC_VERSION}/glibc-${GLIBC_VERSION}.apk \
         https://github.com/sgerrand/alpine-pkg-glibc/releases/download/${GLIBC_VERSION}/glibc-bin-${GLIBC_VERSION}.apk \
         https://github.com/sgerrand/alpine-pkg-glibc/releases/download/${GLIBC_VERSION}/glibc-i18n-${GLIBC_VERSION}.apk && \
    apk add --no-cache glibc-${GLIBC_VERSION}.apk glibc-bin-${GLIBC_VERSION}.apk glibc-i18n-${GLIBC_VERSION}.apk && \
    rm glibc-${GLIBC_VERSION}.apk glibc-bin-${GLIBC_VERSION}.apk glibc-i18n-${GLIBC_VERSION}.apk && \
    /usr/glibc-compat/bin/localedef -i en_US -f UTF-8 en_US.UTF-8 && \
    apk --no-cache add jq bash file && \
    wget https://github.com/freshautomations/sconfig/releases/download/v0.1.0/sconfig_linux_amd64 \
         -O /usr/bin/sconfig && \
    chmod 755 /usr/bin/sconfig && \
    addgroup tendermint && \
    adduser -S -G tendermint tendermint -h "$TM_HOME" && \
    cd /tmp && \
    wget "https://github.com/tendermint/tendermint/releases/download/v${TM_VERSION}/tendermint_${TM_VERSION}_linux_amd64.tar.gz" \
        -O tendermint.tar.gz && \
    echo "${TM_ARCHIVE_HASH}  tendermint.tar.gz" > checksum.txt && \
    sha256sum -c checksum.txt && \
    tar xf tendermint.tar.gz && \
    mv tendermint /usr/bin/tendermint && \
    rm /tmp/checksum.txt && \
    rm /tmp/tendermint.tar.gz && \
    chown -R tendermint:tendermint ${TM_HOME} && \
    mkdir -p /var/log/abci && \
    chown tendermint:tendermint /var/log/abci
USER tendermint
WORKDIR $TM_HOME

EXPOSE 26656 26657 26658 26660
STOPSIGNAL SIGTERM

COPY entrypoint /usr/bin/entrypoint
ENTRYPOINT ["/usr/bin/entrypoint"]
VOLUME [ "$TM_HOME", "/abci", "/var/log/abci" ]
