# ================================================================
# Raptor Launcher — Build & Deploy Automation
# ================================================================

# -------- Configuration --------

ifeq ($(shell uname),Darwin)
    BIN_DIR := $(HOME)/Software/docker-env/crawler
else
    BIN_DIR := $(HOME)/crawler-app
endif


TARGET_DIR    := target/release
BINARIES      := crawler mobile_de raptor-agent

# Database connection string (used during build or runtime)
include makefile.env
export $(shell sed -n 's/^\(.*\)=.*/\1/p' makefile.env)

# -------- Commands --------
CARGO         := cargo
RM            := rm -rf
MKDIR         := mkdir -p
CP            := cp
STRIP         := strip

# -------- Colors --------
GREEN := \033[0;32m
YELLOW := \033[1;33m
BLUE := \033[1;34m
RESET := \033[0m

# -------- Targets --------
.PHONY: all build deploy clean info

# Default target
all: info build deploy

# ---------------------------------------------------------------
# Print environment info
# ---------------------------------------------------------------
info:
	@echo ""
	@echo "$(BLUE)==> Environment Configuration$(RESET)"
	@echo "DATABASE_URL = $(DATABASE_URL)"
	@echo "BIN_DIR      = $(BIN_DIR)"
	@echo ""

# ---------------------------------------------------------------
# Build Rust binaries
# ---------------------------------------------------------------
build:
	@echo "$(YELLOW)==> Building Rust binaries...$(RESET)"
	@$(CARGO) build --release
	@echo "$(GREEN)✅ Build completed successfully.$(RESET)"

# ---------------------------------------------------------------
# Deploy binaries to $(BIN_DIR)
# ---------------------------------------------------------------
deploy:
	@echo "$(YELLOW)==> Deploying binaries to $(BIN_DIR)...$(RESET)"
	@$(MKDIR) $(BIN_DIR)
	@for bin in $(BINARIES); do \
		if [ -f "$(TARGET_DIR)/$$bin" ]; then \
			echo "  Copying $$bin -> $(BIN_DIR)/$$bin"; \
			$(CP) $(TARGET_DIR)/$$bin $(BIN_DIR)/; \
			$(STRIP) $(BIN_DIR)/$$bin 2>/dev/null || true; \
		else \
			echo "  ⚠️  Warning: $(TARGET_DIR)/$$bin not found."; \
		fi \
	done
	@echo "  Copying config directory..."
	@$(CP) -r config $(BIN_DIR)/
	@echo "  Creating empty data directory..."
	@$(MKDIR) $(BIN_DIR)/data
	@echo "$(GREEN)🚀 Deploy completed.$(RESET)"

# ---------------------------------------------------------------
# Clean build artifacts and binaries
# ---------------------------------------------------------------
clean:
	@echo "$(YELLOW)==> Cleaning project...$(RESET)"
	@$(CARGO) clean
	@$(RM) $(BIN_DIR)
	@echo "$(GREEN)🧹 Clean completed.$(RESET)"