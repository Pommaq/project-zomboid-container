A simple container that allows one to quickly get started with running a project zomboid dedicated server on Linux.

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

I recommend using screen if you're hosting this using systemd, since the server binary isnt optimized for headless
use as of pre 42 (see https://theindiestone.com/forums/index.php?/topic/63563-4178-multiplayer-zomboid-dedicated-server-does-not-handle-sigterm/).
Remember to run "quit" in the interactive terminal before killing the container to ensure the files are saved properly due to this 
to avoid save and game corruption.

Remember that you will be prompted for admin account password the first time a new save is started.

You can modify the options the game is called with by setting ``JAVA_OPTS`` environment variable.
