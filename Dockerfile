
# Build our wrapper
FROM docker.io/rust:1-alpine as builder
WORKDIR /wrapper
COPY ./tools/runner .
RUN apk add alpine-sdk
RUN cargo build --release

# Create the actual container we will use
FROM docker.io/steamcmd/steamcmd:centos
# For run_server64.sh
ENV INSTDIR=/install_dir/

# Ensure save files are stored in /saves
RUN mkdir -p /root/Zomboid
RUN ln -s /root/Zomboid /saves


WORKDIR /tools
COPY --from=builder /wrapper/target/release/runner .
COPY ./tools/entrypoint.sh .
RUN chmod +x ./entrypoint.sh
# These are the ports this container uses
EXPOSE "16261" "16261/udp"
EXPOSE "16262" "16262/udp"

# Should probably not be touched
ENV STARTUP_SH_PATH="/install_dir/start-server.sh"

# These can and should be customized
# Sets custom parameters for the server. Each flag and/or value must be comma (,) delimited.
# see https://pzwiki.net/wiki/Startup_parameters for available parameters.
ENV CUSTOM_SERVER_PARAMETERS="-adminpassword,CHANGEME"
# Sets minimum + maximum allowed ram usage by default, allows overriding server JVM parameters
# Set wrapper loglevel, the server itself ignores this.
ENV RUST_LOG="info"

CMD ["/tools/runner"]
ENTRYPOINT ["/tools/entrypoint.sh"]