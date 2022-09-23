FROM alpine:3.16
LABEL maintainer="hello@informal.systems"
ARG TMVERSION

ENV TMHOME=/tendermint
#GLIBC for Alpine from: https://github.com/sgerrand/alpine-pkg-glibc
RUN wget https://alpine-pkgs.sgerrand.com/sgerrand.rsa.pub \
         -O /etc/apk/keys/sgerrand.rsa.pub && \
    wget https://github.com/sgerrand/alpine-pkg-glibc/releases/download/2.32-r0/glibc-2.32-r0.apk \
         https://github.com/sgerrand/alpine-pkg-glibc/releases/download/2.32-r0/glibc-bin-2.32-r0.apk \
         https://github.com/sgerrand/alpine-pkg-glibc/releases/download/2.32-r0/glibc-i18n-2.32-r0.apk && \
    apk add --no-cache glibc-2.32-r0.apk glibc-bin-2.32-r0.apk glibc-i18n-2.32-r0.apk && \
    rm glibc-2.32-r0.apk glibc-bin-2.32-r0.apk glibc-i18n-2.32-r0.apk && \
    /usr/glibc-compat/bin/localedef -i en_US -f UTF-8 en_US.UTF-8 && \
    apk --no-cache add jq bash file && \
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

EXPOSE 26656 26657 26658 26660
STOPSIGNAL SIGTERM

COPY entrypoint /usr/bin/entrypoint
ENTRYPOINT ["/usr/bin/entrypoint"]
VOLUME [ "$TMHOME", "/abci" ]
