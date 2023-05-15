FROM rust:1.66 AS build
RUN apt update
RUN apt install llvm -y
RUN apt install cmake -y
RUN apt install clang -y
RUN apt install libclang-dev -y
ENV LLVM_LIB_DIR /usr/lib/llvm-11/lib
RUN cargo install c2rust
RUN apt install bear -y
RUN rustup override set nightly-2021-11-22-x86_64-unknown-linux-gnu
RUN rustup component add rustfmt --toolchain nightly-2021-11-22-x86_64-unknown-linux-gnu
ARG CACHEBUST=0
COPY . /crusts
RUN cd /crusts \
    && cargo install --path .
RUN install_crown.sh
RUN cd ..
WORKDIR /mnt
ENTRYPOINT [ "crusts" ]
