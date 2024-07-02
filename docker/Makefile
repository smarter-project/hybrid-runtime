PLATFORM = linux/arm64
RUNTIME_ROOT = ../runtime/runtime
WORKDIR = /runtime
VOLUME_RUNTIME = -v $(abspath $(RUNTIME_ROOT)):$(WORKDIR)
SHIM_ROOT = ../runtime/shim
VOLUME_SHIM = -v $(abspath $(SHIM_ROOT)):$(WORKDIR)
RUNTIME_DOCKERFILE = runtime.dockerfile
RUNTIME_IMAGE = runtime-image
BUILD_CMD = cargo build --target aarch64-unknown-linux-musl
IMAGE_DOCKERFILE= image.dockerfile
IMAGE_NAME = hello_world_imx8mp:latest
GITHUB= ghcr.io/smarter-project/hybrid-runtime


image: image.dockerfile
	docker build  ${BUILD_ARGS} -t $(GITHUB)/${IMAGE_NAME} -f ${IMAGE_DOCKERFILE} .

runtime-build: runtime.dockerfile
	docker build -t $(RUNTIME_IMAGE) -f $(RUNTIME_DOCKERFILE) .

runtime-run: runtime-build
	docker run --init --rm $(VOLUME_RUNTIME) -w $(WORKDIR) $(RUNTIME_IMAGE) $(BUILD_CMD)

shim: runtime.dockerfile
	docker run --init --rm $(VOLUME_SHIM) -w $(WORKDIR) $(RUNTIME_IMAGE) $(BUILD_CMD)

all: runtime-run shim

