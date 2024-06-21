FROM ubuntu:22.04 as buildenv

RUN apt update && \
    apt install --no-install-recommends -y \
    build-essential \
    cmake \
    gcc-arm-none-eabi \
    libnewlib-arm-none-eabi \
    libnewlib-dev \
    libstdc++-arm-none-eabi-dev \
    libstdc++-arm-none-eabi-newlib

RUN apt install -y git
RUN apt install -y python3 python3-pip
RUN pip3 install west


RUN mkdir /mcuxsdk
WORKDIR /mcuxsdk
RUN  west init -m https://github.com/NXPmicro/mcux-sdk --mr MCUX_2.15.000 /mcuxsdk
RUN west update
     
WORKDIR /mcuxsdk/examples/evkmimx8mp/demo_apps/hello_world/armgcc

RUN export ARMGCC_DIR=/usr/ && sh ./build_release.sh

FROM scratch as firmware
COPY --from=buildenv /mcuxsdk/examples/evkmimx8mp/demo_apps/hello_world/armgcc/release/hello_world.elf  /hello_world.elf
ENTRYPOINT [ "/hello_world.elf" ]
LABEL board="NXP i.MX8MPlus EVK board" mcu="imx-rproc"


