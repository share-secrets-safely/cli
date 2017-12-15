EXE=target/debug/s3

help:
	$(info Available Targets)
	$(info ---------------------------------------------------------------------------------------------------------------)
	$(info journey-tests    | Run journey tests using a pre-built linux binary)
	$(info - Development -------------------------------------------------------------------------------------------------)
	$(info lint-scripts     | Run journey tests using a pre-built linux binary)
	$(info ---------------------------------------------------------------------------------------------------------------)

$(EXE): 
	cargo build --all-features

journey-tests: $(EXE)
	tests/stateless-journey-test.sh $(EXE)

lint-scripts:
	find . -not -path '*target/*' -name '*.sh' -type f | while read -r sf; do shellcheck -x "$$sf"; done
