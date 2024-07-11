# Running container using hybrid runtime standalone

[Docs on the `hybrid-cli` CLI](./CLI.md)

Follow the instructions in [build instructions](./build_instructions.md) to build both a container firmware image and the CLI.

- Pull the image using containerd cli `ctr`
```
# ctr image pull ghcr.io/smarter-project/hybrid-runtime/hello_world_imx8mp:latest test
```
- Make sure the image is on the board
```
# ctr image ls

REF                                                                                TYPE                                                 DIGEST                                                                  SIZE     PLATFORMS   LABELS 
ghcr.io/smarter-project/hybrid-runtime/hello_world_imx8mp:latest application/vnd.docker.distribution.manifest.v2+json sha256:4643087e5c578ff8804470912fad0fd6f9bef657e9949d43070f404e882ac25c 28.7 KiB linux/arm64 - 
```

- Create a hybrid container
```
# ./hybrid-cli create ghcr.io/smarter-project/hybrid-runtime/hello_world_imx8mp:latest test_container
```

- Make sure the container was created
```
# ctr container ls 

CONTAINER         IMAGE                                                                                 RUNTIME    
test_container    ghcr.io/smarter-project/hybrid-runtime/hello_world_imx8mp:latest               hybrid    
```

- Start the container
```
# ./hybrid-cli start test_container

start container Start { container_id: "test_container" }
mcupath: "/sys/class/remoteproc/remoteproc0"
Microprocessor is offline.
state offline => starting firmware...
Microprocessor is running.
container started
```

- Check container info from containerd
```
# ctr container info test_container
{
    "ID": "test_container",
    "Labels": {
        "Board": "FSL i.MX8MPlus EVK board\u0000",
        "Firmware": "hello_world.elf",
        "MCU": "imx-rproc"
    },
    "Image": "ghcr.io/smarter-project/hybrid-runtime/hello_world_imx8mp:latest",
    "Runtime": {
        "Name": "hybrid",
        "Options": null
    },
    "Spec": {
        "type_url": "types.containerd.io/opencontainers/runtime-spec/1/Spec"
    },
    "SnapshotKey": "temp_key",
    "Snapshotter": "temp_snapshotter",
    "CreatedAt": "2024-03-06T11:06:25.195508587Z",
    "UpdatedAt": "2024-03-06T11:06:25.195508587Z",
    "Extensions": {},
    "SandboxID": ""
}
```

- Check the logs of the running container
```
# ./hybrid-cli logs test_container
```

- Delete the container
```
# ./hybrid-cli delete test_container
```

- Make sure the container was deleted
```
# ctr container ls

CONTAINER    IMAGE    RUNTIME 
```
