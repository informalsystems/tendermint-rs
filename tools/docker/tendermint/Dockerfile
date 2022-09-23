FROM alpine:3.16
LABEL maintainer="hello@informal.systems"
ARG TMVERSION

ENV TMHOME=/tendermint
RUN apk --no-cache add jq bash && \
    wget https://github.com/freshautomations/sconfig/releases/download/v0.1.0/sconfig_linux_amd64 \
        -O /usr/bin/sconfig && \
    chmod 755 /usr/bin/sconfig && \
    cd /tmp && \
    wget https://github.com/tendermint/tendermint/releases/download/v${TMVERSION}/tendermint_${TMVERSION}_linux_amd64.tar.gz \
        -O ./tendermint.tar.gz && \
    tar xf tendermint.tar.gz && \
    mv tendermint /usr/bin/tendermint && \
    chmod 755 /usr/bin/tendermint && \
    rm tendermint.tar.gz && \
    addgroup tendermint && \
    adduser -S -G tendermint tendermint -h "$TMHOME"

USER tendermint
WORKDIR $TMHOME

EXPOSE 26656 26657 26660
STOPSIGNAL SIGTERM

COPY entrypoint /usr/bin/entrypoint
ENTRYPOINT ["/usr/bin/entrypoint"]
CMD ["node"]
VOLUME [ "$TMHOME" ]
