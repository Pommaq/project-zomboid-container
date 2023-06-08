# Zomboid SigTerm Wrapper
This is a neat wrapper around project zomboid which ensures it exits gracefully upon receiving a sigterm signal,
meaning it becomes easier to run as a service on servers with e.g. systemd. It is recommended to run this as a docker or podman container.

This exists since Zomboid doesnt handle termination signals gracefully as of pre 42 (https://theindiestone.com/forums/index.php?/topic/63563-4178-multiplayer-zomboid-dedicated-server-does-not-handle-sigterm/). Prompting the wiki to recommend unreliable hacks
where one begs the server to save, waits for 15 seconds and PRAYS it had time to save. This wrapper will exit ONLY when said server has exited cleanly.

# Configuring server settings
### and adding mods etc.
Once the server is installed atleast once, open the savefile location (e.g. /home/timmy/Zomboid) or Documents/Zomboid
then edit ./Server/servertest.ini to add your settings.


Game is installed and saves are stored at the following locations inside the container by default
```shell
/install_dir
/saves
```

Thus you can build and run the container with persistent server install and saves like the following:
```shell
podman build --tag=zomboid-server .
podman run --rm -it -v $PWD/game_files:/install_dir:Z -v $PWD/saves:/saves:Z -p 16261:16261/udp zomboid-server
```
