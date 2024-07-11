# Build Firmware container image + Hybrid Runtime CLI + Shim

- Navigate to docker folder, [image.dockerfile](../docker/image.dockerfile) is a Dockerfile that builds a hello world app on top of freeRTOS. To build the image, [docker](https://docs.docker.com/engine/install/ubuntu/) and buildx (we're building an arm container image on an x86 machine) are required:


Build a hello-world container firmware image: 

```
# make -C docker image 
```


Build the hybrid-runtime CLI and shim: 

```
# make -C docker all 
```

- The command builds 3 things:
    - Docker firmware image (if you'are not building the image on the board / model directly you need to store the image in a tar file, send it to the board and finally unpack it there).
    - CLI + Runtime (single binary): `hybrid-cli` under `runtime/runtime/target/aarch64-unknown-linux-musl/debug/hyrbid-cli`.
    - Runtime + Shim (single binary): `containerd-shim-containerd-hybrid` under `runtime/shim/target/aarch64-unknown-linux-musl/debug/containerd-shim-containerd-hybrid`.


- Copy the `hybrid-cli` CLI + `containerd-shim-containerd-hybrid` to the board using `scp`: 
```
# scp runtime/runtime/target/aarch64-unknown-linux-musl/debug/hybrid-cli root@{IP of board / model}:/usr/local/bin/

# scp runtime/shim/target/aarch64-unknown-linux-musl/debug/containerd-shim-containerd-hybrid root@imx8m-mini-2.lab.cambridge.arm.com:/usr/local/bin/
```

- In case there is a need to build the firmware using a different approach (other than using the nxp sdk to build freeRTOS) or based on zephyr, just copy the firmware elf file to a scratch image like the following:

> COPY the elf file either from another image using `COPY --from=image_containing_firmware location/firmware.elf /firmware.elf` or copy it from local machine `COPY location/firmware.elf /firmware.elf`.

Dockerfile should look something like this for the i.MX8M-Mini EVK board

```
FROM scratch
COPY --from=build /sdk/boards/evkmimx8mm/demo_apps/release/hello_world.elf /hello_world.elf
ENTRYPOINT [ "hello_world.elf" ]
LABEL board="FSL i.MX8MM EVK board" mcu="imx-rproc"
```

Dockerfile should look something like this for the i.MX8M-Plus EVK model on AVH
```
FROM scratch as firmware
COPY --from=buildenv /mcuxsdk/examples/evkmimx8mp/demo_apps/hello_world/armgcc/release/hello_world.elf  /hello_world.elf
ENTRYPOINT [ "/hello_world.elf" ]
LABEL board="NXP i.MX8MPlus EVK board" mcu="imx-rproc"
```


We rely on 3 information in the Dockerfile: make sure you have the right labels
- Copy the firmware elf file to a scratch image.
- Entrypoint should be set for the name of the firmware.
- Labels: 
    - board="{the string under `/sys/firmware/devicetree/base/model`}"
    - mcu="{name under remoteproc}"
