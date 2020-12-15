FROM alpine:3.12.1
LABEL maintainer="hello@informal.systems"

ENV HOME=/gaiad
ENV GAIAD_CONFIG=$HOME/.gaia

RUN apk --no-cache add jq bash && \
    wget https://github.com/freshautomations/sconfig/releases/download/v0.1.0/sconfig_linux_amd64 \
         -O /usr/bin/sconfig && \
    chmod 755 /usr/bin/sconfig && \
    addgroup gaiad && \
    adduser -S -G gaiad gaiad -h "$HOME"
USER gaiad
WORKDIR $HOME

EXPOSE 1317 26656 26657 26660
STOPSIGNAL SIGTERM

ARG GAIAD=gaiad
ARG GAIAD_CONFIG_FILES_DIR=n0
COPY $GAIAD /usr/bin/gaiad
COPY --chown=gaiad $GAIAD_CONFIG_FILES_DIR $GAIAD_CONFIG

ENTRYPOINT ["/usr/bin/gaiad"]
CMD ["start"]
VOLUME [ "$HOME" ]
