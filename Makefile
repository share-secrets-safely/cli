EXE=target/debug/s3
RELEASE_EXE=target/release/s3
RELEASE_MUSL_EXE=target/x86_64-unknown-linux-musl/release/s3
MUSL_EXE=target/x86_64-unknown-linux-musl/debug/s3
LIBC_EXE=target/x86_64-unknown-linux-gnu/debug/s3
MUSL_IMAGE=clux/muslrust:stable
MY_LIBC_IMAGE=s3_libc:stable
MY_MUSL_IMAGE=s3_musl:stable
CARGO_CACHE_ARGS=-v $$PWD/.docker-cargo-cache:/usr/local/cargo/registry

help:
	$(info Available Targets)
	$(info - Testing -----------------------------------------------------------------------------------------------------)
	$(info lint-scripts           | Run journey tests using a pre-built linux binary)
	$(info journey-tests           | Run all journey tests using a pre-built binary)
	$(info stateful-journey-tests  | Run only stateful journeys in docker)
	$(info stateless-journey-tests | Run only stateless journey)
	$(info - Deployment  -------------------------------------------------------------------------------------------------)
	$(info tag-release            | Create a new release commit using the version in VERSION file)
	$(info deployable-linux       | Archive usable for any more recent linux system)
	$(info deployable-host        | Archive usable on your host)
	$(info - Docker ------------------------------------------------------------------------------------------------------)
	$(info build-linux-musl       | Build the binary via a docker based musl container)
	$(info build-linux-libc       | Build the binary via a docker based libc container)
	$(info build-musl-image       | Build our musl build image)
	$(info build-libc-image       | Build our libc build image)
	$(info interactive-linux-musl | The interactive version of the above (MUSL))
	$(info interactive-linux-libc | The interactive version of the above (libc))
	$(info ---------------------------------------------------------------------------------------------------------------)

always:

$(EXE): always
	cargo build --all-features

$(RELEASE_EXE): always
	cargo build --all-features --release
	
$(MUSL_EXE): build-linux-musl
	
$(RELEASE_MUSL_EXE): release-linux-musl

deployable-linux: $(RELEASE_MUSL_EXE)
	tar czf s3-cli-linux-musl-x86_64.tar.gz -C $(dir $<) $(notdir $<)

deployable-host: $(RELEASE_EXE)
	tar czf s3-cli-$$(uname -s)-$$(uname -m).tar.gz -C $(dir $<) $(notdir $<)
	
$(LIBC_EXE): build-linux-libc
	
tag-release: bin/tag-release.sh release.md VERSION
	bin/tag-release.sh $$(cat VERSION) release.md

stateful-journey-tests: $(MUSL_EXE)
	tests/stateful-journey-test.sh $< $(MY_MUSL_IMAGE)

stateless-journey-tests: $(EXE)
	tests/stateless-journey-test.sh $<

journey-tests: stateless-journey-tests stateful-journey-tests

build-libc-image:
	docker build -t $(MY_LIBC_IMAGE) - < etc/docker/Dockerfile.rust

build-musl-image:
	docker build -t $(MY_MUSL_IMAGE) - < etc/docker/Dockerfile.musl-build

build-linux-musl: build-musl-image
	docker run -v $$PWD/.docker-cargo-cache:/root/.cargo -v "$$PWD:/volume" --rm -it $(MY_MUSL_IMAGE) cargo build --target=x86_64-unknown-linux-musl
	
release-linux-musl: build-musl-image
	docker run -v $$PWD/.docker-cargo-cache:/root/.cargo -v "$$PWD:/volume" --rm -it $(MY_MUSL_IMAGE) cargo build --target=x86_64-unknown-linux-musl --release

build-linux-libc: build-libc-image
	docker run $(CARGO_CACHE_ARGS) -v "$$PWD:/volume" -w '/volume' --rm -it $(MY_LIBC_IMAGE) cargo build --target=x86_64-unknown-linux-gnu
	
interactive-linux-musl: build-musl-image
	docker run -v $$PWD/.docker-cargo-cache:/root/.cargo -v "$$PWD:/volume" --rm -it $(MY_MUSL_IMAGE)

interactive-linux-libc: build-libc-image
	docker run $(CARGO_CACHE_ARGS) -v "$$PWD:/volume" -w '/volume' --rm -it $(MY_LIBC_IMAGE)
	
lint-scripts:
	find . -not \( -path '*target/*' -or -path "*cargo*" \) -name '*.sh' -type f | while read -r sf; do shellcheck -x "$$sf"; done
