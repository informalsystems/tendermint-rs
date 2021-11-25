FROM alpine:3.15
LABEL maintainer="hello@informal.systems"

ARG TM_VERSION=0.35.0
ARG TM_ARCHIVE_HASH=c70dc4538991183905c1eef17263b713666675a995d154a75a52cf0022338724
ENV TM_HOME=/tendermint

RUN apk --no-cache add jq bash && \
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
    chown -R tendermint:tendermint ${TM_HOME}
USER tendermint
WORKDIR $TM_HOME

EXPOSE 26656 26657 26660
STOPSIGNAL SIGTERM

COPY entrypoint /usr/bin/entrypoint
ENTRYPOINT ["/usr/bin/entrypoint"]
CMD ["node"]
VOLUME [ "$TM_HOME" ]
