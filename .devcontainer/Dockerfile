FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Asian/Shanghai

RUN apt-get update \
  && apt-get upgrade -y \
  && apt-get install --no-install-recommends -y clang llvm cmake

#
# wasm-tools
ARG WASM_TOOLS_VER=1.205.0

WORKDIR /opt
RUN wget https://github.com/bytecodealliance/wasm-tools/releases/download/v${WASM_TOOLS_VER}/wasm-tools-${WASM_TOOLS_VER}-x86_64-linux.tar.gz
RUN tar -xvf wasm-tools-${WASM_TOOLS_VER}-x86_64-linux.tar.gz \
  && mv wasm-tools-${WASM_TOOLS_VER}-x86_64-linux wasm-tools \
  && rm wasm-tools-${WASM_TOOLS_VER}-x86_64-linux.tar.gz \
  && ln -s /opt/wasm-tools/wasm-tools /usr/local/bin/wasm-tools
