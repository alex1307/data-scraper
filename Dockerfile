# Start with a rust alpine image
FROM rust:1.76.0-alpine
# This is important, see https://github.com/rust-lang/docker-rust/issues/85
ENV RUSTFLAGS="-C target-feature=-crt-static"
# if needed, add additional dependencies here
RUN apk add --no-cache musl-dev
RUN apk add --no-cache openssl-dev
RUN apk add --no-cache pkgconfig
RUN apk update && apk add bash
RUN apk add --no-cache gcc
RUN apk --no-cache add \
    bash \
    g++ \
    ca-certificates \
    lz4-dev \
    musl-dev \
    cyrus-sasl-dev \
    openssl-dev \
    make \
    python3 \
    protoc 
# set the workdir and copy the source into it
WORKDIR /app
COPY ./ /app
# do a release build
RUN cargo build --release
RUN strip target/release/crawler

# use a plain alpine image, the alpine version needs to match the builder
FROM alpine:latest
# if needed, install additional dependencies here
RUN apk add --no-cache libgcc
ENV RUST_LOG=info
ENV KAFKA_BROKER=localhost:9092
# copy the binary into the final image
COPY --from=0 /app/target/release/crawler .
RUN mkdir -p resources/searches
RUN mkdir -p _log
RUN mkdir -p config
COPY --from=0 /app/config ./config
COPY --from=0 /app/resources/searches ./resources/searches
RUN echo "Current directory:" && pwd
RUN echo "Current directory:" && ls -la
RUN echo "Contents of current directory:" && ls -la /config
RUN echo "Contents of resources directory:" && ls -la /resources
RUN echo "Contents of searches directory:" && ls -la /resources/searches
# set the binary as entrypoint
ENTRYPOINT ["/crawler"]