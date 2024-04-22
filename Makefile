# Builds and runs the privatemail

CGO     := cargo
NAME    := privatemail
SRCDIR  := .

.PHONY: all
all: test

# Default target runs all
default: all

.PHONY: fmt
fmt:
	$(CGO) fmt

.PHONY: lint
lint:
	$(CGO) install cargo-audit --features=fix
	$(CGO) audit fix
	$(CGO) clippy --fix --allow-staged --allow-dirty

.PHONY: build
build: fmt lint
	$(CGO) build

.PHONY: clean
clean:
	$(CGO) clean

.PHONY: test
test: build
	$(CGO) test

.PHONY: publish
publish:
	@echo "** WARNING: Publishing requires a valid API token!**"
	@echo "** Please set the '${CARGO_API_TOKEN}' environment variable before publishing. **"
	$(CGO) build --release
	$(CGO) publish --quiet

.PHONY: deploy
deploy: publish
	@echo "** WARNING: Deploying requires AWS credentials... **"
	@bash release.sh
