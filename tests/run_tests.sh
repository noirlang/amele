#!/usr/bin/env bash
# ==============================================================================
# Worm Forensic Tool - Premium Test Runner
# ==============================================================================

# Set styling colors
BOLD="\033[1m"
GREEN="\033[38;5;46m"
BLUE="\033[38;5;39m"
RED="\033[38;5;196m"
YELLOW="\033[38;5;220m"
CYAN="\033[38;5;51m"
RESET="\033[0m"

# Header
echo -e "${CYAN}======================================================================${RESET}"
echo -e "${BOLD}${CYAN}      🐛 Worm Forensic Tool - Unified Diagnostic & Test Suite${RESET}"
echo -e "${CYAN}======================================================================${RESET}"
echo -e "${BLUE}Starting full verification of Backend, Frontend, and Translation assets...${RESET}"
echo ""

# Get the script directory and navigate to the project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT" || exit 1

# Track overall success
BACKEND_PASS=0
I18N_PASS=0
ROUTES_PASS=0

# Phase 1: Rust Backend Unit Tests
echo -e "${BOLD}${BLUE}📦 [1/3] Running Rust Backend Unit Tests...${RESET}"
echo -e "${CYAN}------------------------------------------------${RESET}"
if cargo test; then
  echo -e "\n${GREEN}✔ Rust backend tests completed successfully!${RESET}"
  BACKEND_PASS=1
else
  echo -e "\n${RED}✘ Rust backend tests failed! Please check cargo warnings/errors.${RESET}"
fi
echo ""

# Phase 2: Frontend Translation Key Check
echo -e "${BOLD}${BLUE}🌐 [2/3] Running Translation Dictionary Tests...${RESET}"
echo -e "${CYAN}------------------------------------------------${RESET}"
if node --test tests/i18n.test.js; then
  echo -e "${GREEN}✔ Translation dictionaries are perfectly synchronized!${RESET}"
  I18N_PASS=1
else
  echo -e "${RED}✘ Translation diagnostic failed! Mismatched keys or placeholders.${RESET}"
fi
echo ""

# Phase 3: Frontend ESM Modules & Router Check
echo -e "${BOLD}${BLUE}🖥️ [3/3] Running ES Modules & Route Dispatcher Tests...${RESET}"
echo -e "${CYAN}------------------------------------------------${RESET}"
if node --test tests/routes.test.js; then
  echo -e "${GREEN}✔ ES Modules and routing dispatcher loaded flawlessly!${RESET}"
  ROUTES_PASS=1
else
  echo -e "${RED}✘ ES Modules check failed! Syntax or ReferenceError detected.${RESET}"
fi
echo ""

# Final Summary Report
echo -e "${CYAN}======================================================================${RESET}"
echo -e "${BOLD}${CYAN}                     Verification Summary Report${RESET}"
echo -e "${CYAN}======================================================================${RESET}"

if [ $BACKEND_PASS -eq 1 ]; then
  echo -e "  [${GREEN}PASS${RESET}]  Phase 1: Rust Backend Unit Tests"
else
  echo -e "  [${RED}FAIL${RESET}]  Phase 1: Rust Backend Unit Tests"
fi

if [ $I18N_PASS -eq 1 ]; then
  echo -e "  [${GREEN}PASS${RESET}]  Phase 2: i18n Translation Key Integrity"
else
  echo -e "  [${RED}FAIL${RESET}]  Phase 2: i18n Translation Key Integrity"
fi

if [ $ROUTES_PASS -eq 1 ]; then
  echo -e "  [${GREEN}PASS${RESET}]  Phase 3: Frontend ES Modules & Routes Dispatcher"
else
  echo -e "  [${RED}FAIL${RESET}]  Phase 3: Frontend ES Modules & Routes Dispatcher"
fi

echo -e "${CYAN}----------------------------------------------------------------------${RESET}"

if [ $BACKEND_PASS -eq 1 ] && [ $I18N_PASS -eq 1 ] && [ $ROUTES_PASS -eq 1 ]; then
  echo -e "${BOLD}${GREEN}✨ ALL SYSTEMS NOMINAL: Worm Forensic Tool is 100% ready for deployment!${RESET}"
  exit 0
else
  echo -e "${BOLD}${RED}⚠️ DIAGNOSTIC WARNING: Some verification stages failed. Please resolve them.${RESET}"
  exit 1
fi
