## Build Stage
# Pull base image and update
FROM ekidd/rust-musl-builder:stable AS builder

USER root

# RUN git clone https://github.com/Riverbend-High-School/gabe_versus_gavin.git /app
COPY . /app

# Move to repo
WORKDIR /app

# Build app
RUN cargo build --release --target x86_64-unknown-linux-musl

## Final Stage
# Pull final image and copy binary
FROM alpine:latest AS final

WORKDIR /app

# Copy our build
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/rhs_mailer /app/rhs_mailer
COPY --from=builder /app/entrypoint.sh /app

# Expose web http port
EXPOSE 9999

ENTRYPOINT ["sh", "/app/entrypoint.sh"]