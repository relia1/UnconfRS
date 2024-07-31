ARG APP_NAME=thesis
FROM rust:latest AS build
WORKDIR /app

# Cache downloaded+built dependencies
COPY *.toml /app/
RUN \
    mkdir /app/src && \
    mkdir /app/templates && \
    echo 'fn main() {}' > /app/src/main.rs && \
    cargo build --release && \
    rm -Rvf /repo/src

# Build our actual code
COPY src /app/src
COPY templates /app/templates
COPY migrations /app/migrations
RUN \
    touch src/main.rs && \
    cargo build --release

FROM rust:latest AS final
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
COPY --from=build /app/target/release/Thesis /bin/
RUN ls -l
# COPY --chown=appuser:appuser ./assets ./assets
#COPY --chown=appuser:appuser migrations/ /migrations/
# Expose the port that the application listens on.
EXPOSE 3000

# What the container should run when it is started.
CMD ["/bin/Thesis"]
