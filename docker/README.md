## Build firmware container image

As an example we use the nxp sdk to build a freeRTOS firmware, the sdk also has couple of demo apps, we'll be using the hello world demo example.

The goal here is to have a lightweight OCI container image that contains a firmware, to obtain this:

- First, we need access to the nxp sdk from a Dockerfile, to download the sdk we need to login to the nxp website, to make this simple, we created a docker image with the sdk already in it and pushed it to our internal gitlab [image with sdk]( git.research.arm.com:4567/attk/hybrid/containers/nxp_sdk_imx8mm). This is done in [sdk.dockerfile](./sdk.dockerfile)
- Second, we need to build the firmware, I've coppied the sdk from the previous image to an ubnutu one and built the firmware.
- Finally, we package the firmware into an empty scratch container image.
- The image is created using 2 labels, `board="FSL i.MX8MM EVK board" mcu="imx-rproc"`

`/sys/firmware/devicetree/base/model` => `FSL i.MX8MM EVK board`
under `/sys/firmware/devicetree/base/` there is an `imx8mm-cm4` .. file `name` contains `imx8mm-cm4`

- We use labels while building the image to specify which board / SoC / MCU the firmware is for, this will be needed to match the image to the right board. To build an image with labels in docker just add `LABEL` instruction to dockerfile followed by the key-value pair, for example, `LABEL board=imx8m-mini mcu=cortex-m4`. We match these labels with information on the board. [Labels documentation](https://docs.docker.com/reference/dockerfile/#label).

This info can be retrieved under `/sys/firmware/devicetree/base/compatible`
```sh
model = "FSL i.MX8MM EVK board";
compatible = "fsl,imx8mm-evk\0fsl,imx8mm";
```

example Dockerfile would look something like: 
```sh
FROM scratch
COPY --from=build /sdk/boards/evkmimx8mm/demo_apps/release/hello_world.elf /hello_world.elf
LABEL board="FSL i.MX8MM EVK board" mcu="imx-rproc"
ENTRYPOINT [ "hello_world.elf" ]
```


To build the image, run `make image`, to make sure a `imx8mm_fregit.research.arm.com:4567/attk/hybrid/containers/imx8mm_freertos:latest` was create, use: `docker images`.


## Build hybrid CLI (To interact with the runtime using the hybrid CLI)

Run `make runtime-run`. A `hybrid-cli` binary should be built under `../runtime/hybrid-runtime/target/aarch64-unknown-linux-musl/debug/`

Next step would be to copy the runtime to your board via `scp` for example.


## Build hybrid shim (To interact with the runtime using k3s or containerd)

Run `make shim`. A `containerd-shim-containerd-hybrid` binary should be built under `../runtime/hybrid-shim/target/aarch64-unknown-linux-musl/debug/`. Next copy the shim binary to the board.

- Build Docker image, runtime and shim binary at once, run `make all`.

> PS: the `hybrid-cli` is a CLI to interact with the runtime, you'd either use the CLI or the shim, not both, since they accomplish the same thing.