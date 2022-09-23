FROM alpine:3.16
LABEL maintainer="hello@informal.systems"

ENV TMHOME=/tendermint
RUN apk --no-cache add jq bash && \
    wget https://github.com/freshautomations/sconfig/releases/download/v0.1.0/sconfig_linux_amd64 \
         -O /usr/bin/sconfig && \
    chmod 755 /usr/bin/sconfig && \
    addgroup tendermint && \
    adduser -S -G tendermint tendermint -h "$TMHOME"
USER tendermint
WORKDIR $TMHOME

EXPOSE 26656 26657 26660
STOPSIGNAL SIGTERM

ARG TENDERMINT=tendermint
COPY $TENDERMINT /usr/bin/tendermint

COPY entrypoint /usr/bin/entrypoint
ENTRYPOINT ["/usr/bin/entrypoint"]
CMD ["node"]
VOLUME [ "$TMHOME" ]
