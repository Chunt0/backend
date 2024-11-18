# Builder stage
FROM rust:1.81.0 AS builder

WORKDIR /app
RUN apt-get update && apt-get install lld clang -y
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install OpenSSL - it is dynamically linked by some of our dependencies 
# Install ca-certificates - it is needed to verify TLS certs when 
# establishing an HTTPS connection
RUN apt-get update -y \
	&& apt-get install -y --no-install-recommends openssl ca-certificates \
	# Clean up
	&& apt-get autoremove -y \
	&& apt-get clean -y \
	&& rm -rf /var/lib/apt/lists/*

# Copy the compile binary from the builder env to our runtime env
COPY --from=builder /app/target/release/backend backend
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./backend"]
