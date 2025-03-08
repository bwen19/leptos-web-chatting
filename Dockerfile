# stage 1 - create chef container
FROM rust:1.80.0-slim-bullseye AS chef

RUN apt-get update -y && apt-get install -y curl
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
RUN apt-get install -y nodejs

RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked cargo-chef
WORKDIR /app

# stage 2 - generate a rust recipe file for dependencies
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# stage 3 - build our dependencies
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

RUN npm install -g tailwindcss
RUN cargo install --locked cargo-leptos

COPY . .
ENV LEPTOS_TAILWIND_VERSION="v3.4.10"
RUN cargo leptos build --release -vv

# stage 4 - create runtime container
FROM gcr.io/distroless/cc-debian11 AS runtime
WORKDIR /app

COPY --from=builder /app/target/release/server .
COPY --from=builder /app/target/site ./site

COPY --from=builder /app/common/migrations .
COPY --from=builder /app/db/chat.db ./db/

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ROOT=/app/site
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
EXPOSE 8080

CMD [ "/app/server" ]