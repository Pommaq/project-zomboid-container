
# Build our wrapper
FROM docker.io/rust:1-alpine as builder
WORKDIR /wrapper
COPY ./runner .
RUN apk add alpine-sdk
RUN cargo build --release

FROM docker.io/steamcmd/steamcmd:centos
# Sets minimum + maximum allowed ram usage
ENV JAVA_OPTS="-Xms1g -Xmx8g"
# For run_server64.sh
ENV INSTDIR=/install_dir/

ARG PASSWORD="CHANGEME"
# Ensure save files are stored in /saves
RUN mkdir -p /root/Zomboid
RUN ln -s /root/Zomboid /saves

WORKDIR /tools
COPY --from=builder /wrapper/target/release/runner .
ENTRYPOINT steamcmd +force_install_dir /install_dir +login anonymous +app_update 380870 +quit; /tools/runner /install_dir/start-server.sh $PASSWORD
