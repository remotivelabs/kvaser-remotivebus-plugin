# Kvaser Remotivebus Plugin Service

This package provides a systemd service that runs the `kvaser-remotivebus-plugin` binary.

## Getting started

```bash
# build and install the kvaser-remotivebus-plugin service
make install

# The service will be automatically enabled and started during installation
# If needed, you can manually control it with:
sudo systemctl start kvaser-remotivebus-plugin.service
sudo systemctl stop kvaser-remotivebus-plugin.service
sudo systemctl status kvaser-remotivebus-plugin.service
```

Removing the service:

```bash
make uninstall
```
