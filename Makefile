EXE=target/debug/sy
RELEASE_EXE=target/release/sy
RELEASE_MUSL_EXE=target/x86_64-unknown-linux-musl/release/sy
MUSL_EXE=target/x86_64-unknown-linux-musl/debug/sy
MUSL_IMAGE=clux/muslrust:stable
MY_MUSL_IMAGE=sheesy_musl:stable
MY_LINUX_RUN_IMAGE=alpine_with_gpg2:stable
CARGO_CACHE_ARGS=-v $$PWD/.docker-cargo-cache:/usr/local/cargo/registry
MUSL_DOCKER_ARGS=docker run -v $$PWD/.docker-cargo-cache:/root/.cargo -v "$$PWD:/volume" --rm -it $(MY_MUSL_IMAGE)
HOST_DEPLOYABLE=sy-cli-$$(uname -s)-$$(uname -m).tar.gz
LINUX_DEPLOYABLE=sy-cli-linux-musl-x86_64.tar.gz

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
	$(info deployment             | Archive usable on your host)
	$(info - Docker ------------------------------------------------------------------------------------------------------)
	$(info build-linux-musl       | Build the binary via a docker based musl container)
	$(info build-musl-image       | Build our musl build image)
	$(info interactive-linux-musl | The interactive version of the above (MUSL))
	$(info ---------------------------------------------------------------------------------------------------------------)

always:

$(EXE): always
	cargo build

$(RELEASE_EXE): always
	cargo build --release

$(MUSL_EXE): build-linux-musl

$(RELEASE_MUSL_EXE): release-linux-musl

deployable-linux: $(RELEASE_MUSL_EXE)
	$(MUSL_DOCKER_ARGS) strip --strip-all $<
	gpg --yes --output $<.gpg --detach-sig $<
	tar czf $(LINUX_DEPLOYABLE) -C $(dir $<) $(notdir $<) $(notdir $<).gpg

deployable-host: $(RELEASE_EXE)
	strip $<
	gpg --yes --output $<.gpg --detach-sig $<
	tar czf $(HOST_DEPLOYABLE) -C $(dir $<) $(notdir $<) $(notdir $<).gpg

tag-release: bin/tag-release.sh release.md VERSION
	bin/tag-release.sh $$(cat VERSION) release.md

stateful-journey-tests: $(MUSL_EXE) build-linux-run-image
	tests/stateful-journey-test.sh $< $(MY_LINUX_RUN_IMAGE)

stateless-journey-tests: $(EXE)
	tests/stateless-journey-test.sh $<

journey-tests: stateful-journey-tests stateless-journey-tests

deployment: lint-scripts journey-tests
	$(MAKE) deployable-host
	$(MAKE) deployable-linux

build-linux-run-image:
	docker build -t $(MY_LINUX_RUN_IMAGE) - < etc/docker/Dockerfile.alpine-gpg2

build-musl-image:
	docker build -t $(MY_MUSL_IMAGE) - < etc/docker/Dockerfile.musl-build

build-linux-musl: build-musl-image
	$(MUSL_DOCKER_ARGS) cargo build --target=x86_64-unknown-linux-musl

release-linux-musl: build-musl-image
	docker run -v $$PWD/.docker-cargo-cache:/root/.cargo -v "$$PWD:/volume" --rm -it $(MY_MUSL_IMAGE) cargo build --target=x86_64-unknown-linux-musl --release

interactive-linux-musl: build-musl-image
	docker run -v $$PWD/.docker-cargo-cache:/root/.cargo -v "$$PWD:/volume" -it $(MY_MUSL_IMAGE)

lint-scripts:
	find . -not \( -path '*target/*' -or -path "*cargo*" \) -name '*.sh' -type f | while read -r sf; do shellcheck -x "$$sf"; done
