# Hybrid Runtime CLI

## Name

hybrid-runtime - a OCI runtime for hybrid systems.

## Description

hybrid-runtime is a CLI for running embedded apps on hybrid systems (on microcontrollers) that follows the Open Container Initiative format.

## Synopsis

The runtime must support commands with the following templates:

`hybrid-runtime [global-options] <COMMAND> [command-specific-options] <command-specific-arguments>`

- OCI compatibility, the runtime should implement the [OCI runtime specification](https://github.com/opencontainers/runtime-spec), this means it should idealy implement the following commands. however, we are restricted by what is possible in remoteproc (lifecycle management for remote processors)

    - `create`: creates a container.
    - `start`: starts a container.
    - `delete`: deletes a container. ==> stops the firmware running and from docker/k3s side deletes the container.
    - `logs`: Fetch the logs of a running container.
`resume`, `run`, `spec`, `update`, `pause` and `version` are out of scope for this runtime.

- [Image spec](https://github.com/opencontainers/image-spec/blob/main/image-index.md).


## Hybrid Runtime CLI

Run `hybrid-runtime`

```
# ./hybrid-runtime 

Deploy an application across available embedded cores.

Usage: hybrid-runtime <COMMAND>

Commands:
  create  Create a container
  start   Start a created container
  state   Output the state of a container
  kill    terminates the container
  delete  Deletes any resources held by the container
  logs    Fetch the logs of a running container
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Create

- Create container: we can either provide the path to a local firmware or the name of a docker image, both followed by a unique container ID of your choosing. Run `hybrid-runtime create --help`

```
# ./hybrid-runtime create --help
Create a container

Usage: hybrid-runtime create <IMAGE> <CONTAINER_ID>

Arguments:
  <IMAGE>         Firmware container image name
  <CONTAINER_ID>  container ID name for the instance of the container that you are starting. The name you provide for the container instance must be unique on your host

Options:
  -h, --help                 Print help
```

### Start

- Start: start a created container

```
# ./hybrid-runtime start --help
Start a created container

Usage: hybrid-runtime start <CONTAINER_ID>

Arguments:
  <CONTAINER_ID>  container ID

Options:
  -h, --help  Print help
```

### Logs

- Logs: Fetch the logs of a  running container

```
# ./hybrid-runtime logs --help
Fetch the logs of a running container

Usage: hybrid-runtime logs <CONTAINER_ID>

Arguments:
  <CONTAINER_ID>  Container ID Name of the container

Options:
  -h, --help  Print help
```

### Delete

- Delete: deletes a container

```
# ./hybrid-runtime delete
Deletes any resources held by the container

Usage: hybrid-runtime delete [OPTIONS] <CONTAINER_ID>

Arguments:
  <CONTAINER_ID>  container ID

Options:
  -f, --force  Forcibly deletes the container if it is still running (using SIGKILL)
  -h, --help   Print help
```

