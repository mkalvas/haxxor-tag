FROM rust:latest as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --bin server --path .

# Strip final image to bare necessities
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*

ENV HAXXOR_HOST="0.0.0.0:3000"
ENV HAXXOR_URL="http://0.0.0.0:3000"

COPY --from=builder /usr/local/cargo/bin/server /usr/local/bin/server

# pid 1 needs to be something other than the server so that sigint is sent
# properly, so we wrap in sh
CMD ["/bin/sh", "-c", "server"]

# docker build . -t haxxor-tag:latest
# docker run -it --rm -p 3000:3000 haxxor-tag:latest
# HAXXOR_URL="http://0.0.0.0:3000" cargo run --bin tui
