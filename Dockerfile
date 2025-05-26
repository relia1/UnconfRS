ARG APP_NAME=unconfrs
FROM rust:slim-bookworm AS build
ARG BUILD_TYPE
WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Cache downloaded+built dependencies
COPY *.toml /app/
RUN if [ "$BUILD_TYPE" = "release" ]; then \
    mkdir /app/src && \
    mkdir /app/src/bin && \
    mkdir /app/web && \
    echo 'fn main() {}' > /app/src/main.rs && \
    echo 'fn main() {}' > /app/src/bin/test_unconf.rs && \
    cargo build --release && \
    rm -Rvf /app/src target/${BUILD_TYPE}/deps/unconfrs*; \
else \
    mkdir /app/src && \
    mkdir /app/src/bin && \
    mkdir /app/web && \
    echo 'fn main() {}' > /app/src/main.rs && \
    echo 'fn main() {}' > /app/src/bin/test_unconf.rs && \
    cargo build && \
    rm -Rvf /app/src target/${BUILD_TYPE}/deps/unconfrs*; \
fi

# Build our actual code
COPY src /app/src
COPY web /app/web
COPY migrations /app/migrations
RUN if [ "$BUILD_TYPE" = "release" ]; then \
    touch src/main.rs && \
    cargo build --release; \
else \
    touch src/main.rs && \
    cargo build; \
fi

FROM debian:bookworm-slim AS final
ARG BUILD_TYPE
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/go/dockerfile-user-best-practices/
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy the executable from the "build" stage.
COPY --from=build /app/target/${BUILD_TYPE}/unconfrs /bin/
COPY --from=build /app/web/ /
RUN ls -l
# COPY --chown=appuser:appuser ./assets ./assets
#COPY --chown=appuser:appuser migrations/ /migrations/
# Expose the port that the application listens on.
EXPOSE 3000

# What the container should run when it is started.
CMD ["/bin/unconfrs"]
