ARG APP_NAME=unconfrs
FROM rust:slim-bookworm AS build
ARG BUILD_TYPE
WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Cache downloaded+built dependencies
COPY Cargo.toml /app/
COPY server/*.toml /app/server/
COPY scheduler/Cargo.toml /app/scheduler/
COPY test_unconf/Cargo.toml /app/test_unconf/
COPY .sqlx /app/.sqlx

RUN mkdir -p /app/server/src && \
    mkdir -p /app/scheduler/src && \
    mkdir -p /app/scheduler/src/bin && \
    mkdir -p /app/test_unconf/src && \
    echo 'fn main() {}' > /app/server/src/main.rs && \
    echo 'pub fn main() {}' > /app/scheduler/src/lib.rs && \
    echo 'fn main() {}' > /app/scheduler/src/bin/scheduler_eval.rs && \
    echo 'fn main() {}' > /app/test_unconf/src/main.rs

RUN if [ "$BUILD_TYPE" = "release" ]; then \
    cargo build --release -p server; \
else \
    cargo build -p server; \
fi

RUN rm -rf /app/server/src /app/scheduler/src /app/test_unconf/src;

COPY server/src /app/server/src
COPY test_unconf/src /app/test_unconf/src
COPY scheduler/src /app/scheduler/src
COPY server/web /app/server/web
COPY server/migrations /app/server/migrations

RUN if [ "$BUILD_TYPE" = "release" ]; then \
    touch server/src/main.rs && \
    touch scheduler/src/lib.rs && \
    cargo build --release -p server; \
else \
    touch server/src/main.rs && \
    touch scheduler/src/lib.rs && \
    cargo build -p server; \
fi

FROM debian:stable-slim AS final
ARG BUILD_TYPE
RUN apt-get update && apt-get install -y \
    ca-certificates \
    postgresql \
    postgresql-contrib \
    sudo \
    && rm -rf /var/lib/apt/lists/*;
# Create app user and postgres data directory
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/home/appuser" \
    --shell "/bin/bash" \
    --uid "${UID}" \
    appuser && \
    mkdir -p /var/lib/postgresql/data /var/run/postgresql && \
    chown -R appuser:appuser /var/lib/postgresql /var/run/postgresql /etc/postgresql

# Copy the executable and migrations from the "build" stage.
COPY --from=build /app/target/release/server /bin/
COPY --from=build /app/server/web/ /
COPY --from=build /app/server/migrations/ /migrations/

# Create startup script
COPY startup.sh /startup.sh
RUN chmod +x /startup.sh

USER appuser
# Expose the port that the application listens on.
EXPOSE 3039

# What the container should run when it is started.
CMD ["/startup.sh"]
