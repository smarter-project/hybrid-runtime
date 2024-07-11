# Running container using hybrid runtime + containerd

### Build runtime 

Follow the instructions in [build instructions](./build_instructions.md) to build both a container firmware image and the runtime.

Copy the `containerd-shim-containerd-hybrid` to the board under `/usr/local/bin` using `scp` for example: 
```
# scp runtime/shim/target/aarch64-unknown-linux-musl/debug/containerd-shim-containerd-hybrid root@{IP address}:/usr/local/bin/
```

>  The name of the runtime should follow the name convention set by OCI `io.containerd.{name}.{version}`. In our case, it's `io.containerd.hybrid`, it's specified in main fn `shim::run::<Service>("io.containerd.hybrid", None)`

> No need to change anything in containerd configuration since we copied the runtime under `/usr/local/bin`, containerd will look for the runtime there.

- Pull the image using containerd cli `ctr`
```
# ctr image pull ghcr.io/smarter-project/hybrid-runtime/hello_world_imx8mp:latest
```
- Make sure the image is on the board
```
# ctr image ls

REF                                                                                TYPE                                                 DIGEST                                                                  SIZE     PLATFORMS   LABELS 
ghcr.io/smarter-project/hybrid-runtime/hello_world_imx8mp:latest application/vnd.docker.distribution.manifest.v2+json sha256:4643087e5c578ff8804470912fad0fd6f9bef657e9949d43070f404e882ac25c 28.7 KiB linux/arm64 - 
```

- Create and start the container

```
# ctr run --runtime io.containerd.hybrid ghcr.io/smarter-project/hybrid-runtime/hello_world_imx8mp:latest test
```

- Check the container was created
```
# ctr c ls

CONTAINER    IMAGE                                                                      RUNTIME                 
test        ghcr.io/smarter-project/hybrid-runtime/hello_world_imx8mp:latest    io.containerd.hybrid 
```

- Check container info

```
# ctr c info test

{
    "ID": "test",
    "Labels": {
        "Board": "FSL i.MX8MM EVK board",
        "Firmware name": "hello_world.elf",
        "Firmware path": "/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs/snapshots/180/fs",
        "MCU name": "imx-rproc",
        "MCU path": "/arm/containers/runtime/hybrid-shim/remoteproc/remoteproc0"
    },
    "Image": "ghcr.io/smarter-project/hybrid-runtime/hello_world_imx8mp:latest",
    "Runtime": {
        "Name": "io.containerd.hybrid",
        "Options": null
    }, 
    ...... 
}
```
- Check the container is running

```
# ctr t ls
TASK     PID       STATUS    
test    451135    RUNNING
```

- Stop container

```
# ctr t kill test
```

- Check container was stopped

```
# ctr t ls

TASK     PID       STATUS    
test    451684    STOPPED
```


- Delete container

```
# ctr c rm test
```

For Debugging, use `journalctl -f -u containerd`.
   