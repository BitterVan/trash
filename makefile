.PHONY: build
build:
	cargo build

.PHONY: test
test: build
	./target/debug/trash

.PHONY: test_redir
test_redir: build
	./target/debug/trash < test/test.sh

.PHONY: test_file
test_file: build
	./target/debug/trash test/test.sh
