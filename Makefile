EXE=target/debug/s3
MUSL_EXE=target/x86_64-unknown-linux-musl/debug/s3
LIBC_EXE=target/x86_64-unknown-linux-gnu/debug/s3
MUSL_IMAGE=clux/muslrust:stable
MY_LIBC_IMAGE=s3_libc:stable
MY_MUSL_IMAGE=s3_musl:stable
OSX_BREW_LIB_DIR=/usr/local/lib
CONTAINER_LIB_DIR=/usr/lib/x86_64-linux-gnu
CARGO_CACHE_ARGS=-v $$PWD/.docker-cargo-cache:/usr/local/cargo/registry

help:
	$(info Available Targets)
	$(info ---------------------------------------------------------------------------------------------------------------)
	$(info journey-tests           | Run all journey tests using a pre-built binary)
	$(info stateful-journey-tests  | Run only stateful journeys in docker)
	$(info stateless-journey-tests | Run only stateless journey)
	$(info - Development -------------------------------------------------------------------------------------------------)
	$(info build-musl-image       | Build our musl build image)
	$(info build-libc-image       | Build our libc build image)
	$(info build-linux-musl       | Build the binary via a docker based musl container)
	$(info build-linux-libc       | Build the binary via a docker based libc container)
	$(info clean-linux-musl       | Runs cargo clean in the musl container)
	$(info clean-linux-libc       | Runs cargo clean in the libc container)
	$(info interactive-linux-musl | The interactive version of the above (MUSL))
	$(info interactive-linux-libc | The interactive version of the above (libc))
	$(info lint-scripts           | Run journey tests using a pre-built linux binary)
	$(info ---------------------------------------------------------------------------------------------------------------)

always:

$(EXE): always
	GPGME_LIB_PATH=$(OSX_BREW_LIB_DIR) DEP_GPG_ERROR_ROOT=x86_64-unknown-linux-musl GPG_ERROR_LIB_PATH=$(OSX_BREW_LIB_DIR) GPG_ERROR_LIBS=gpg-error cargo build --all-features

$(MUSL_EXE): build-linux-musl
	
$(LIBC_EXE): build-linux-libc

stateful-journey-tests: $(LIBC_EXE)
	tests/stateful-journey-test.sh $< $(MY_LIBC_IMAGE)

stateless-journey-tests: $(EXE)
	tests/stateless-journey-test.sh $<

journey-tests: stateless-journey-tests stateful-journey-tests

build-libc-image:
	docker build -t $(MY_LIBC_IMAGE) - < etc/docker/Dockerfile.rust

build-musl-image:
	docker build -t $(MY_MUSL_IMAGE) - < etc/docker/Dockerfile.musl-build

build-linux-musl: build-musl-image
	docker run -e GPGME_LIB_PATH=$(CONTAINER_LIB_DIR) -e DEP_GPG_ERROR_ROOT=x86_64-unknown-linux-musl -e GPG_ERROR_LIB_PATH=$(CONTAINER_LIB_DIR) -e GPG_ERROR_LIBS=gpg-error -v $$PWD/.docker-cargo-cache:/root/.cargo -v "$$PWD:/volume" --rm -it $(MY_MUSL_IMAGE) cargo build --target=x86_64-unknown-linux-musl

build-linux-libc: build-libc-image
	docker run $(CARGO_CACHE_ARGS) -v "$$PWD:/volume" -w '/volume' --rm -it $(MY_LIBC_IMAGE) cargo build --target=x86_64-unknown-linux-gnu
	
clean-linux-musl: build-musl-image
	docker run -v $$PWD/.docker-cargo-cache:/root/.cargo -v "$$PWD:/volume" --rm -it $(MY_MUSL_IMAGE) cargo clean

clean-linux-libc: build-libc-image
	docker run $(CARGO_CACHE_ARGS) -v "$$PWD:/volume" -w '/volume' --rm -it $(MY_LIBC_IMAGE) cargo clean
	
interactive-linux-musl: build-musl-image
	docker run -v $$PWD/.docker-cargo-cache:/root/.cargo -v "$$PWD:/volume" --rm -it $(MY_MUSL_IMAGE)

interactive-linux-libc: build-libc-image
	docker run $(CARGO_CACHE_ARGS) -v "$$PWD:/volume" -w '/volume' --rm -it $(MY_LIBC_IMAGE)
	
lint-scripts:
	find . -not \( -path '*target/*' -or -path "*cargo*" \) -name '*.sh' -type f | while read -r sf; do shellcheck -x "$$sf"; done
