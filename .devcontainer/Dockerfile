FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Asian/Shanghai

RUN apt-get update \
  && apt-get upgrade -y \
  && apt-get install --no-install-recommends -y cmake gnupg lsb-release software-properties-common wget

RUN cd /tmp \
  && wget https://apt.llvm.org/llvm.sh \
  && chmod a+x llvm.sh \
  && ./llvm.sh 15

RUN ln -sf /usr/bin/lldb-15 /usr/bin/lldb
