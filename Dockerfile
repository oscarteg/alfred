# Use an official Rust runtime as a parent image
FROM rust:1.73 as builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the current directory contents into the container at /usr/src/app
COPY . .

# Build the Rust application in release mode
RUN cargo build --release

# Use a minimal image to run
FROM debian:buster-slim

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/app/target/release/alfred /usr/local/bin/alfred

# Command to run the application
CMD ["alfred"]
