FROM rust:slim-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
	# required for vscode server:
    git \
    curl \
    ca-certificates \
	# cleanup:
    && rm -rf /var/lib/apt/lists/*

