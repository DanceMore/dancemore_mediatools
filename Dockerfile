# Use the official Rust image as the base image
FROM rust:1.86 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --package tv_mode_web

### stage 2
# Create a new lightweight image with just the binary
FROM debian:bookworm-slim

# Set the working directory inside the container
WORKDIR /app

# Copy the binary from the builder stage to the final image
COPY --from=builder /app/target/release/tv_mode_web /app/tv_mode_web

ENV ROCKET_ADDRESS="0.0.0.0"
ENV ROCKET_PROFILE="production"

# Expose the port your Rocket server will listen on (change to your port)
EXPOSE 8000

# Command to run your Rocket application
CMD ["/app/tv_mode_web"]
