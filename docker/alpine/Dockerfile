# Simple dockerfile example to build a quibitous and start in genesis mode

FROM alpine:3.9.4
LABEL MAINTAINER TBCO
LABEL description="Quibitous"

ARG VERSION=latest
ARG PREFIX=/app
ARG REST_PORT=8448

ENV ENV_PREFIX=${PREFIX}

# prepare the environment
RUN apk add --no-cache curl jq

# download the release
RUN if [ "${VERSION}" = "latest" ] ; then tag_name="$(curl --silent https://api.github.com/repos/the-blockchain-company/quibitous/releases/latest | jq -r .tag_name)" ; else tag_name="v${VERSION}" ; fi && \
    mkdir -p ${ENV_PREFIX}/src && \
    mkdir -p ${ENV_PREFIX}/bin && \
    cd ${ENV_PREFIX}/bin && \
    curl -L "https://github.com/the-blockchain-company/quibitous/releases/download/${tag_name}/quibitous-${tag_name}-x86_64-unknown-linux-musl.tar.gz" | tar xz && \
    cd ${ENV_PREFIX}/src && \
    curl -L "https://github.com/the-blockchain-company/quibitous/archive/${tag_name}.tar.gz" | tar xz && \
    cp */scripts/* ${ENV_PREFIX}/bin/ && \
    rm -r ${ENV_PREFIX}/src

ENV PATH=${ENV_PREFIX}/bin:${PATH}
WORKDIR ${ENV_PREFIX}/bin

RUN mkdir -p ${ENV_PREFIX}/cfg && \
    mkdir -p ${ENV_PREFIX}/secret && \
    sh ./bootstrap -p ${REST_PORT} -x -c ${ENV_PREFIX}/cfg -k ${ENV_PREFIX}/secret

EXPOSE ${REST_PORT}

CMD [ "sh", "startup_script.sh" ]
