# Containers and Orchestration

`hybrid-runtime` is an OCI compatible container runtime for deploying and running workloads on hybrid systems, boards with an SoC containing a Cortex-A plus a Cortex-M/R using cloud tools such as k3s and containerd.

There are 3 different ways to use the runtime, either standalone meaning interacting with the runtime directly using a CLI without using k3s or containerd, or using containerd's CLI `ctr`, and finally using k3s.

The guides were tested on both an AVH model of i.MX8M Plus baord and the i.MX8M Mini board, we recommend using the AVH model model, they offer a 30 day free trial. 

- [AVH model setup](./docs/AVH.md).
- [Build Firmware container image + Hybrid Runtime CLI + Shim](./docs/build_instructions.md).
- [Running container using hybrid runtime CLI](./docs/hybrid-runtime_standalone.md).
- [Docs on the `hybrid-runtime` CLI](./docs/CLI.md)
- [Running container using hybrid runtime + containerd](./docs/hybrid-runtime_containerd.md).
- [Running container using hybrid runtime + k3s](./docs/hybrid-runtime_k3s.md).