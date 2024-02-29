FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Asian/Shanghai

RUN apt-get update \
  && apt-get upgrade -y \
  && apt-get install --no-install-recommends -y cmake

RUN apt-get install --no-install-recommends -y clang-11
