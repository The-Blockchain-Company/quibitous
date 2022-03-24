# Simple dockerfile example to build a quibitous and start in genesis mode

FROM ubuntu:cosmic
LABEL MAINTAINER TBCO
LABEL description="Quibitous latest"

ARG PREFIX=/app
ENV ENV_PREFIX=${PREFIX}
ARG REST_PORT=8448
ARG BUILD=false
ENV ENV_BUILD=${BUILD}
ARG VER=v0.2.3
ENV ENV_VER=${VER}

# prepare the environment
RUN apt-get update && \
    apt-get install -y git curl && \
    mkdir -p ${ENV_PREFIX} && \
    mkdir -p ${ENV_PREFIX}/src && \
    mkdir -p ${ENV_PREFIX}/bin && \
    cd ${ENV_PREFIX} && \
    git clone --recurse-submodules https://github.com/The-Blockchain-Company/quibitous src && \
    cd src && git checkout ${ENV_VER} && \
    cp scripts/bootstrap scripts/faucet-send-money.shtempl scripts/faucet-send-certificate.shtempl \
    scripts/create-stakepool.shtempl \
    scripts/create-account-and-delegate.shtempl scripts/qcli-helpers ${ENV_PREFIX}/bin

# install the node and qcli from binary
RUN if [ "${ENV_BUILD}" = "false" ]; then \
    echo "[INFO] - you have selected to install binaries" && \
    cd ${ENV_PREFIX}/bin && \
    curl -s -O -L https://github.com/The-Blockchain-Company/quibitous/releases/download/${ENV_VER}/quibitous-${ENV_VER}-x86_64-unknown-linux-gnu.tar.gz && \
    tar xzf quibitous-${ENV_VER}-x86_64-unknown-linux-gnu.tar.gz && rm quibitous-${ENV_VER}-x86_64-unknown-linux-gnu.tar.gz ; fi

# install the node and qcli from source
RUN if [ "${ENV_BUILD}" = "true" ]; then \
    echo "[INFO] - you have selected to build and install from source" && \
    apt-get install -y build-essential pkg-config && \
    bash -c "curl https://sh.rustup.rs -sSf | bash -s -- -y" && \
    export PATH=$HOME/.cargo/bin:$PATH && \
    rustup install stable && \
    rustup default stable && \
    cd ${ENV_PREFIX}/src && \
    git submodule update --init --recursive && \
    cargo build --release && \
    cargo install --force --path quibitous && \
    cargo install --force --path qcli && \
    cp $HOME/.cargo/bin/quibitous $HOME/.cargo/bin/qcli ${ENV_PREFIX}/bin && \
    rm -rf $HOME/.cargo $HOME/.rustup ; fi

# cleanup
RUN rm -rf ${ENV_PREFIX}/src && \
    RM_ME=`apt-mark showauto` && \
    apt-get remove --purge --auto-remove -y git curl build-essential pkg-config ${RM_ME} && \
    apt-get install -y --no-install-recommends && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

ENV PATH=${ENV_PREFIX}/bin:${PATH}
WORKDIR ${ENV_PREFIX}/bin

RUN mkdir -p ${ENV_PREFIX}/cfg && \
    mkdir -p ${ENV_PREFIX}/secret && \
    sh ./bootstrap -p ${REST_PORT} -x -c ${ENV_PREFIX}/cfg -k ${ENV_PREFIX}/secret

EXPOSE ${REST_PORT}

CMD [ "sh", "startup_script.sh" ]
