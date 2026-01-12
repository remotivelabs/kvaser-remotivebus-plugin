# Package information
PACKAGE_NAME = kvaser-remotivebus-plugin
VERSION = REPLACE_VERSION_NUMBER

# Package manager detection
DEB_CMD = dpkg
DNF_CMD = dnf
HAS_DEB := $(shell command -v $(DEB_CMD) 2> /dev/null)
HAS_DNF := $(shell command -v $(DNF_CMD) 2> /dev/null)

# Architecture detection
DEB_ARCH := $(shell if [ -n "$(HAS_DEB)" ]; then dpkg --print-architecture; fi)
RPM_ARCH := $(shell if [ -n "$(HAS_DNF)" ]; then rpm --eval '%{_arch}'; fi)

# Package paths
DEB_PKG = deb/$(PACKAGE_NAME)_$(VERSION)_$(DEB_ARCH).deb
RPM_PKG = rpm/$(PACKAGE_NAME)-$(VERSION)-1.$(RPM_ARCH).rpm

# Installation commands
define install_deb
	sudo $(DEB_CMD) -i $(DEB_PKG)
	echo "Successfully installed $(PACKAGE_NAME) v$(VERSION) for $(DEB_ARCH)"
endef

define install_rpm
	sudo $(DNF_CMD) install -y $(RPM_PKG)
	echo "Successfully installed $(PACKAGE_NAME) v$(VERSION) for $(RPM_ARCH)"
endef

# Uninstallation commands
define uninstall_deb
	sudo $(DEB_CMD) -r $(PACKAGE_NAME)
endef

define uninstall_rpm
	sudo $(DNF_CMD) remove -y $(PACKAGE_NAME)
endef

.PHONY: install
install: $(if $(HAS_DEB),install-deb,$(if $(HAS_DNF),install-rpm,install-error))

install-deb:
	@$(install_deb)

install-rpm:
	@$(install_rpm)

install-error:
	@echo "Error: Neither $(DEB_CMD) nor $(DNF_CMD) package manager found" && exit 1

.PHONY: uninstall
uninstall: $(if $(HAS_DEB),uninstall-deb,$(if $(HAS_DNF),uninstall-rpm,uninstall-error))

uninstall-deb:
	@$(uninstall_deb)

uninstall-rpm:
	@$(uninstall_rpm)

uninstall-error:
	@echo "Error: Neither $(DEB_CMD) nor $(DNF_CMD) package manager found" && exit 1
	@echo "Successfully uninstalled $(PACKAGE_NAME)"
