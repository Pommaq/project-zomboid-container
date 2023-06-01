FROM docker.io/steamcmd/steamcmd:centos

# Sets minimum + maximum allowed ram usage
ENV JAVA_OPTS="-Xms1g -Xmx8g"
# For run_server64.sh
ENV INSTDIR=/install_dir/

# Ensure save files are stored in /saves
RUN mkdir -p /root/Zomboid
RUN ln -s /root/Zomboid /saves

WORKDIR /install_dir

ENTRYPOINT steamcmd +force_install_dir /install_dir +login anonymous +app_update 380870 +quit; bash ./start-server.sh
