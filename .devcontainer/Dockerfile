FROM rust:slim-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
	# required for vscode server:
    git \
    curl \
    ca-certificates \
	# cleanup:
    && rm -rf /var/lib/apt/lists/*

# Enable colors (Debian has them but disabled by default)
ENV TERM=xterm-256color
RUN sed -i 's/^#force_color_prompt=yes/force_color_prompt=yes/' /etc/skel/.bashrc
