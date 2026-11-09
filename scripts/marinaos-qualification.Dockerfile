# Test harness only; this is not a MarinaOS distribution image.
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update -qq && apt-get install -y -qq --no-install-recommends systemd util-linux && rm -rf /var/lib/apt/lists/*
