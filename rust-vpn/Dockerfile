# Use the official Rust image as a parent image
FROM rust:1.74

# Install necessary packages
RUN apt-get update && \
    apt-get install -y \
    libssl-dev \
    pkg-config \
    iproute2 \
    sudo \
    && rm -rf /var/lib/apt/lists/*

# Create a new user and add to sudoers
RUN useradd -ms /bin/bash user && echo "user:user" | chpasswd && echo "user ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

# Set the working directory
WORKDIR /usr/src/app

# Copy the source code
COPY . .

# Build the Rust application
RUN cargo build --release

# Expose the port that the server will run on
EXPOSE 12345

# Run the server binary
CMD ["./target/release/rustvpn", "server"]
