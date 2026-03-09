# ╔══════════════════════════════════════════════════════════════════════════════╗
# ║                          SOLVERFORGE-MAIL                                  ║
# ║                   ratatui TUI email client · himalaya backend              ║
# ╚══════════════════════════════════════════════════════════════════════════════╝
#
# Part of SolverForge Linux — https://solverforge.org
#
# Usage:
#   make              Build release binary
#   make dev          Build debug + run
#   make install      Install to /usr/local
#   make help         Show all targets
#

SHELL     := /bin/bash
.DEFAULT_GOAL := release

# ── Project ──────────────────────────────────────────────────────────────────

NAME      := solverforge-mail
VERSION   := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
BIN       := target/release/$(NAME)
BIN_DBG   := target/debug/$(NAME)
HIMALAYA  := /opt/himalaya/target/release/himalaya

# ── Install paths ────────────────────────────────────────────────────────────

PREFIX    ?= /usr/local
BINDIR    := $(PREFIX)/bin
SHAREDIR  := $(PREFIX)/share/$(NAME)

# ── Colors ───────────────────────────────────────────────────────────────────

C_RST     := \033[0m
C_BLD     := \033[1m
C_DIM     := \033[2m
C_GRN     := \033[32m
C_YLW     := \033[33m
C_CYN     := \033[36m
C_RED     := \033[31m
C_MAG     := \033[35m

define BANNER
	@printf '$(C_BLD)$(C_CYN)'
	@printf '  ┌──────────────────────────────────────┐\n'
	@printf '  │  ✉  solverforge-mail %-15s │\n' '$(VERSION)'
	@printf '  └──────────────────────────────────────┘\n'
	@printf '$(C_RST)'
endef

msg = @printf '  $(C_BLD)$(C_GRN)%-10s$(C_RST) %s\n' '$(1)' '$(2)'
warn = @printf '  $(C_BLD)$(C_YLW)%-10s$(C_RST) %s\n' '$(1)' '$(2)'
err = @printf '  $(C_BLD)$(C_RED)%-10s$(C_RST) %s\n' '$(1)' '$(2)'

# ══════════════════════════════════════════════════════════════════════════════
#  BUILD
# ══════════════════════════════════════════════════════════════════════════════

.PHONY: release debug check clippy fmt test clean help
.PHONY: dev run install uninstall setup accounts
.PHONY: deps-check himalaya-check loc info

release: deps-check ## Build optimized release binary
	$(BANNER)
	$(call msg,CARGO,release build)
	@cargo build --release 2>&1 | sed 's/^/    /'
	$(call msg,OK,$(BIN) ($(shell du -h $(BIN) 2>/dev/null | cut -f1 || echo '?'))

debug: ## Build debug binary
	$(call msg,CARGO,debug build)
	@cargo build 2>&1 | sed 's/^/    /'
	$(call msg,OK,$(BIN_DBG))

check: ## Type-check without codegen (fast)
	$(call msg,CHECK,type checking)
	@cargo check 2>&1 | sed 's/^/    /'

# ══════════════════════════════════════════════════════════════════════════════
#  QUALITY
# ══════════════════════════════════════════════════════════════════════════════

clippy: ## Run clippy lints
	$(call msg,CLIPPY,linting)
	@cargo clippy --all-targets -- -D warnings 2>&1 | sed 's/^/    /'

fmt: ## Format code with rustfmt
	$(call msg,FMT,formatting)
	@cargo fmt

fmt-check: ## Check formatting without changes
	$(call msg,FMT,checking)
	@cargo fmt -- --check

test: ## Run all tests
	$(call msg,TEST,running test suite)
	@cargo test 2>&1 | sed 's/^/    /'
	$(call msg,OK,all tests passed)

lint: clippy fmt-check ## Run all lints (clippy + fmt check)

ci: lint test release ## Full CI pipeline: lint, test, build
	$(call msg,CI,all checks passed)

# ══════════════════════════════════════════════════════════════════════════════
#  RUN
# ══════════════════════════════════════════════════════════════════════════════

dev: debug ## Build debug and run
	$(call msg,RUN,debug mode)
	@./$(BIN_DBG)

run: release ## Build release and run
	$(call msg,RUN,release mode)
	@./$(BIN)

run-account: release ## Run with specific account (ACCOUNT=name)
	$(call msg,RUN,account=$(ACCOUNT))
	@./$(BIN) --account $(ACCOUNT)

# ══════════════════════════════════════════════════════════════════════════════
#  INSTALL
# ══════════════════════════════════════════════════════════════════════════════

install: release ## Install to PREFIX (default /usr/local)
	$(BANNER)
	$(call msg,INSTALL,$(BINDIR)/$(NAME))
	@install -Dm755 $(BIN) $(DESTDIR)$(BINDIR)/$(NAME)
	$(call msg,INSTALL,setup scripts → $(SHAREDIR)/)
	@install -d $(DESTDIR)$(SHAREDIR)
	@install -m755 setup-accounts.sh $(DESTDIR)$(SHAREDIR)/
	@install -m755 setup-common.sh $(DESTDIR)$(SHAREDIR)/
	@install -m755 setup-icloud.sh $(DESTDIR)$(SHAREDIR)/
	@install -m755 setup-blinkenshell.sh $(DESTDIR)$(SHAREDIR)/
	@install -m755 setup-oauth.sh $(DESTDIR)$(SHAREDIR)/
	@install -m755 store-passwords.sh $(DESTDIR)$(SHAREDIR)/
	@install -m755 fix-all-accounts.sh $(DESTDIR)$(SHAREDIR)/
	@install -m755 setup.py $(DESTDIR)$(SHAREDIR)/
	$(call msg,OK,installed $(NAME) v$(VERSION))

uninstall: ## Remove installed files
	$(call warn,REMOVE,$(BINDIR)/$(NAME))
	@rm -f $(DESTDIR)$(BINDIR)/$(NAME)
	$(call warn,REMOVE,$(SHAREDIR)/)
	@rm -rf $(DESTDIR)$(SHAREDIR)
	$(call msg,OK,uninstalled)

# ══════════════════════════════════════════════════════════════════════════════
#  SETUP
# ══════════════════════════════════════════════════════════════════════════════

setup: ## Interactive account setup wizard
	$(call msg,SETUP,launching account wizard)
	@python3 setup.py

accounts: ## List configured email accounts
	$(call msg,ACCOUNTS,querying himalaya)
	@$(HIMALAYA) account list 2>/dev/null || \
		($(call err,ERROR,no accounts configured — run 'make setup'); false)

# ══════════════════════════════════════════════════════════════════════════════
#  HOUSEKEEPING
# ══════════════════════════════════════════════════════════════════════════════

clean: ## Remove build artifacts
	$(call warn,CLEAN,removing target/)
	@cargo clean
	$(call msg,OK,clean)

dist-clean: clean ## Clean everything including Cargo.lock
	$(call warn,CLEAN,removing Cargo.lock)
	@rm -f Cargo.lock

# ══════════════════════════════════════════════════════════════════════════════
#  INFO
# ══════════════════════════════════════════════════════════════════════════════

deps-check: ## Verify build dependencies
	@command -v cargo >/dev/null 2>&1 || \
		{ $(call err,MISSING,cargo — install Rust via https://rustup.rs); exit 1; }

himalaya-check: ## Verify himalaya is available
	@test -x $(HIMALAYA) || \
		{ $(call err,MISSING,himalaya at $(HIMALAYA)); exit 1; }
	$(call msg,OK,himalaya found)

loc: ## Count lines of code
	$(BANNER)
	@printf '  $(C_DIM)%-30s %s$(C_RST)\n' 'File' 'Lines'
	@printf '  $(C_DIM)%-30s %s$(C_RST)\n' '──────────────────────────────' '─────'
	@find src -name '*.rs' | sort | while read f; do \
		printf '  %-30s %s\n' "$$f" "$$(wc -l < "$$f")"; \
	done
	@printf '  $(C_DIM)%-30s %s$(C_RST)\n' '──────────────────────────────' '─────'
	@printf '  $(C_BLD)%-30s %s$(C_RST)\n' 'Total' "$$(find src -name '*.rs' -exec cat {} + | wc -l)"

info: ## Show project info
	$(BANNER)
	@printf '  $(C_DIM)version$(C_RST)    %s\n' '$(VERSION)'
	@printf '  $(C_DIM)rustc$(C_RST)      %s\n' "$$(rustc --version 2>/dev/null || echo 'not found')"
	@printf '  $(C_DIM)cargo$(C_RST)      %s\n' "$$(cargo --version 2>/dev/null || echo 'not found')"
	@printf '  $(C_DIM)himalaya$(C_RST)   %s\n' "$$($(HIMALAYA) --version 2>/dev/null || echo 'not found')"
	@printf '  $(C_DIM)binary$(C_RST)     %s\n' '$(BIN)'
	@printf '  $(C_DIM)prefix$(C_RST)     %s\n' '$(PREFIX)'
	@echo

# ══════════════════════════════════════════════════════════════════════════════
#  HELP
# ══════════════════════════════════════════════════════════════════════════════

help: ## Show this help
	$(BANNER)
	@grep -E '^[a-zA-Z_-]+:.*##' $(MAKEFILE_LIST) | \
		awk -F ':.*## ' '{ printf "  $(C_BLD)$(C_CYN)%-14s$(C_RST) %s\n", $$1, $$2 }'
	@echo
	@printf '  $(C_DIM)Examples:$(C_RST)\n'
	@printf '    make                    Build release binary\n'
	@printf '    make dev                Build debug + run\n'
	@printf '    make ci                 Full lint → test → build pipeline\n'
	@printf '    make install            Install to /usr/local\n'
	@printf '    make run-account ACCOUNT=icloud\n'
	@echo
