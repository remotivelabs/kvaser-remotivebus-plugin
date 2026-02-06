# Kvaser RemotiveBus Plugin

Example plugin for [RemotiveBus](http://docs.remotivelabs.com/docs/remotive-bus) enabling LIN over Kvaser Hydra devices.

The plugin has been tested with the following Kvaser adapters:

- https://kvaser.com/product/kvaser-hybrid-pro-can-lin/
- https://kvaser.com/product/kvaser-hybrid-pro-2xcan-lin/

You'll find device drivers for the adapters on the product pages above.

## Getting started

```bash
# getting help
make help

# build packages for all architectures and formats
make all

# ... or build for specific architecture/format type
make pkg-amd64-deb    # Build .deb package for AMD64
make pkg-arm64-rpm    # Build .rpm package for ARM64
```

For development, just use cargo build:

```bash
make build-amd64-kvaser-libs  # Build Kvaser libraries first
cargo build                   # Cargo automatically finds libraries in build/lib/<arch>
```

## RemotiveBus Plugin API implementation

| Property                | Value                             | Notes                        |
|-------------------------|-----------------------------------|------------------------------|
| Plugin ID               | `kvaser`                          | Unique ID for this plugin    |
| Unix Domain Socket path | `/run/remotivebus/plugins/kvaser` | Plugin communication channel |

### RemotiveTopology configuration

Example RemotiveTopology `platform.yaml` configuation:

```yaml
channels:
  lin0:
    type: lin
    baudrate: 19200
    database: rlcm.ldf
```

Example RemotiveTopology `instance.yaml` configuration:

```yaml
channels:
  lin0:
    type: lin
    driver:
      type: remotivebus
      config:
        type: vcan
        device: lin0
        host_device: vlin0
        plugin:
          driver: kvaser
          host_mode: master
          device_id: 011121:1

ecus:
  RLCM:
    channels:
      lin0:
        config:
          type: lin
          schedule_autostart: true
          schedule_table_name: LpcLpc12LinSchedule01
```

See [Documentation](https://docs.remotivelabs.com/docs/remotive-bus) for more information.

### Commands

Example `start` command:

```json
{
  "action": "start",
  "bus": {
    "type": "vcan",
    "host_device": "mylin",
    "plugin": {
        "driver": "kvaser",
        "host_mode": "master",
        "device_id": "011121:1"
    }
  }
}
```

Example `stop` command:

```json
{
  "action": "stop",
  "bus": {
    "type": "vcan",
    "host_device": "mylin",
    "plugin": {
        "driver": "kvaser",
        "host_mode": "master",
        "device_id": "011121:1"
    }
  }
}
```

### Message schema

| Field                     | Type     | Required | Description |
|---------------------------|----------|----------|-------------|
| `version`                 | integer  | no       | RemotiveBus format version. Not used. |
| `action`                  | string   | yes      | Command type. Must be `start` \|`stop`. |
| `bus.type`                | string   | yes      | CAN bus type. Will always be `vcan`. |
| `bus.device`              | string   | no       | Name for CAN device inside Docker. Not used. |
| `bus.host_device`         | string   | yes      | Name of CAN physical device on host machine. |
| `bus.baudrate`            | integer  | no       | CAN baudrate to apply (used if device was down). Defaults to `19200` bps. |
| `bus.baudrate_fd`         | integer  | no       | CAN-FD baudrate to apply. Not used. |
| `bus.txqueuelen`          | integer  | no       | Override network interface tx write buffer for physical devices. Not used. |
| `bus.plugin.driver`       | string   | yes      | Name of plugin. Will always be `kvaser`. |
| `bus.plugin.name`         | string   | no       | LIN interface name used for debugging. Defaults to `bus.host_device`. |
| `bus.plugin.type`         | string   | no       | Plugin run mode. Must be `lin`|`simulator`. Defaults to `lin`. See [Using the simulator](#using-the-simulator). |
| `bus.plugin.host_mode`    | string   | yes      | LIN host mode. Must be `master`|`slave`. |
| `bus.plugin.device_id`    | string   | yes      | LIN device id. Example `011121:1`. |
| `bus.plugin.base_tick_ms` | string   | no       | LIN base tick in milliseconds. Defaults to `5` ms. |

## Build System

The build system uses Docker containers with architecture isolation (`%` replaced by `amd64/arm64`):

- `build-%-plugin`: Build plugin binary for specific architecture
- `build-%-kvaser-libs`: Build Kvaser linuxcan libraries for specific architecture
- `pkg-%-deb/rpm`: Package for specific architecture and format

All builds automatically source environment variables from `.env`.

The build system automatically downloads and builds the Kvaser LinuxCAN libraries.

```bash
make build-amd64-kvaser-libs
make build-arm64-kvaser-libs
```

## Local development

Install dependencies:
```bash
sudo apt update && sudo apt install build-essential clang libclang-dev
```

[Install Rust](https://www.rust-lang.org/tools/install)

```bash
source .env

# Build Kvaser libraries first (required for cargo to link against)
make build-amd64-kvaser-libs  # or build-arm64-kvaser-libs for ARM64

# Build the Rust application (finds libraries automatically in build/lib/<arch>)
cargo build

# format
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings

# test
cargo test

# start the server
cargo run -- -l debug -p /tmp/kvaser.sock

# stop the server
cargo run -- -p /tmp/kvaser.sock

# clean
cargo clean -q
```

Optionally, run against preinstalled kvaser libraries (usually installs in `/usr/lib`):

```bash
# Override default library path with `KVASER_LIB_PATH`
KVASER_LIB_PATH=/usr/lib cargo run -- -l debug -p /tmp/kvaser.sock
```

### Using the simulator

It is possible to have the plugin run as a LIN simulator, both as master and slave.

Running the simulator from topology:

```yaml
channels:
  lin0:
    type: lin
    driver:
      type: remotivebus
      config:
        type: vcan
        device: myvlin
        host_device: lin0
        plugin:
          driver: kvaser
          name: MyVLIN
          type: simulator
          host_mode: slave
          database: simulator/simulator.ldf,
          schedule_table_name: DEVMLIN01Schedule01
```

There is also a util for sending `start`/`stop` messages to a running plugin:

```bash
# start
cargo run --bin send-msg -- -p /tmp/kvaser.sock -m simulator/start.json

# stop
cargo run --bin send-msg -- -p /tmp/kvaser.sock -m simulator/stop.json
```
