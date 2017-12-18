EXE=target/debug/s3
MUSL_EXE=target/x86_64-unknown-linux-musl/debug/s3
MUSL_IMAGE=clux/muslrust:stable
MY_MUSL_IMAGE=s3_muslrust:stable
OSX_BREW_LIB_DIR=/usr/local/lib
CONTAINER_LIB_DIR=/usr/lib/x86_64-linux-gnu

help:
	$(info Available Targets)
	$(info ---------------------------------------------------------------------------------------------------------------)
	$(info journey-tests           | Run all journey tests using a pre-built binary)
	$(info stateful-journey-tests  | Run only stateful journeys in docker)
	$(info stateless-journey-tests | Run only stateless journey)
	$(info - Development -------------------------------------------------------------------------------------------------)
	$(info build-image            | Build our build image)
	$(info lint-scripts           | Run journey tests using a pre-built linux binary)
	$(info build-linux-musl       | Build the binary via a docker based musl container)
	$(info clean-linux-musl       | Runs cargo clean in the container)
	$(info interactice-linux-musl | The interactive version of the above)
	$(info ---------------------------------------------------------------------------------------------------------------)

always:

$(EXE): always
	GPGME_LIB_PATH=$(OSX_BREW_LIB_DIR) DEP_GPG_ERROR_ROOT=x86_64-unknown-linux-musl GPG_ERROR_LIB_PATH=$(OSX_BREW_LIB_DIR) GPG_ERROR_LIBS=gpg-error GPG_ERROR_STATIC=1 GPGME_STATIC=1 cargo build --all-features

$(MUSL_EXE): build-linux-musl

stateful-journey-tests: $(MUSL_EXE)
	tests/stateful-journey-test.sh $< $(MUSL_IMAGE)

stateless-journey-tests: $(EXE)
	tests/stateless-journey-test.sh $<

journey-tests: stateless-journey-tests stateful-journey-tests

build-image:
	docker build -t $(MY_MUSL_IMAGE) - < etc/docker/Dockerfile.musl-build

build-linux-musl: build-image
	docker run -e GPGME_LIB_PATH=$(CONTAINER_LIB_DIR) -e DEP_GPG_ERROR_ROOT=x86_64-unknown-linux-musl -e GPG_ERROR_LIB_PATH=$(CONTAINER_LIB_DIR) -e GPG_ERROR_LIBS=gpg-error -e GPG_ERROR_STATIC=1 -e GPGME_STATIC=1 -v $$PWD/.docker-cargo-cache:/root/.cargo -v "$$PWD:/volume" --rm -it $(MY_MUSL_IMAGE) cargo build

clean-linux-musl: build-image
	docker run -v $$PWD/.docker-cargo-cache:/root/.cargo -v "$$PWD:/volume" --rm -it $(MY_MUSL_IMAGE) cargo clean

interactive-linux-musl: build-image
	docker run -v $$PWD/.docker-cargo-cache:/root/.cargo -v "$$PWD:/volume" --rm -it $(MY_MUSL_IMAGE)

lint-scripts:
	find . -not \( -path '*target/*' -or -path "*cargo*" \) -name '*.sh' -type f | while read -r sf; do shellcheck -x "$$sf"; done
