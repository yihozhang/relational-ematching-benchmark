FROM docker.io/library/rust:1.55-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    time make python3-matplotlib 
WORKDIR /usr/src/app

# this will force a registry download
RUN cargo search egg