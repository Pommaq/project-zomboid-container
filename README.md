# Zomboid SigTerm Wrapper
This is a neat wrapper around project zomboid which ensures it exits gracefully upon receiving a sigterm signal,
meaning it becomes easier to run as a service on servers with e.g. systemd. It is recommended to run this as a docker or podman container.

This exists since Zomboid doesnt handle termination signals gracefully as of pre 42 (https://theindiestone.com/forums/index.php?/topic/63563-4178-multiplayer-zomboid-dedicated-server-does-not-handle-sigterm/). Prompting the wiki to recommend [unreliable hacks](https://pzwiki.net/wiki/Dedicated_Server#System.d)
where one begs the server to save, waits for 15 seconds and PRAYS it had time to save. This wrapper will exit ONLY when said server has exited cleanly.

# How to use
## Passing parameters
You can pass parameters directly to the zomboid server in two ways.
Firstly using the `CUSTOM_SERVER_PARAMETERS` environment variable, or directly as an argument to the executable. Both scenarios require the parameters to be passed as one singular string, with each parameter separated by commas (,). I suggest reading the Dockerfile for details.

## Installing a server using Docker
The container will install the latest version of project zomboid using steamcmd, with the game files stored inside the container at `/install_dir`, saves at `/saves` and logs at `/logs` by default. 

Thus you can build and run the container with persistent server install, saves and logs like the following:
```shell
podman run --rm -it -v $PWD/game_files:/install_dir -v $PWD/saves:/saves -v $PWD/logs:/logs -p 16261:16261/udp -p 16262:16262/udp -p 27015:27015/tcp ghcr.io/pommaq/project-zomboid-container:master
```
Or for a more permanent install, run it as a systemd service:
```
[Unit]
Description=Project Zomboid Server
After=containerd.service
Wants=network-online.target

[Service]
TimeoutSec=900
Restart=always
ExecStartPre=-/usr/bin/docker  stop -i zomboid

ExecStart=/usr/bin/docker  run --rm --pull=always --name zomboid -v /root/game_files:/install_dir -v /root/save_files:/saves -v /root/logs:/logs -p 16261:16261/udp -p 16262:16262/udp -p 27015:27015/tcp -e CUSTOM_SERVER_PARAMETERS="-adminpassword,password123,-Xms1024m,-Xmx8192m" ghcr.io/pommaq/project-zomboid-container:master

ExecStop=/bin/docker stop zomboid

[Install]
WantedBy=multi-user.target
```

## Standalone
For parameter details, run the binary with --help.
```shell
cd tools/runner
cargo run -- --help
# (OR if the runner is installed and added to $PATH)
runner --help
```

First you will need to manually [install zomboid](https://pzwiki.net/wiki/Dedicated_Server#Through_SteamCMD) and the wrapper located at `tools/runner`.
Then, assuming zomboid was installed at /install_dir:
```shell
# Set admin password using parameters to ensure the server won't prompt us
runner /install_dir/start-server.sh "'-adminpassword,MYCOOLPASSWORD'"
```

Or as a systemd service, assuming runner was installed as `/usr/bin/runner`

```
[Unit]
Description=Project Zomboid Server
Wants=network-online.target

[Service]
TimeoutSec=900
Restart=always
ExecStart=/usr/bin/runner /install_dir/start-server.sh "'-adminpassword,password123,-Xms1024m,-Xmx8192m'"


[Install]
WantedBy=multi-user.target
```


# Configuring Zomboid server
### and adding mods etc.
Once the server is installed open the savefile location (e.g. /home/timmy/Zomboid) or Documents/Zomboid where the configuration files will be located as [per usual for zomboid](https://pzwiki.net/wiki/Dedicated_Server#Customizing_Settings).



