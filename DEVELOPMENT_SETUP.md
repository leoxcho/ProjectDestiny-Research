# Project Destiny Development Setup

## Overview

This document describes how to set up a local development environment for Project Destiny.

Project Destiny is developed primarily in Rust with supporting Python research tools.

---

# Requirements

## Rust

Install the Rust toolchain:

https://rustup.rs/

Verify installation:

```bash
rustc --version
cargo --version
```

---

## Python

Python 3 is required for supporting research utilities.

Verify:

```bash
python3 --version
```

---

# Clone Repository

Clone the repository:

```bash
git clone https://github.com/leoxcho/ProjectDestiny-Research.git
```

Enter the project directory:

```bash
cd ProjectDestiny-Research
```

---

# Build Project

Build the Rust workspace:

```bash
cargo build
```

Build an optimized release version:

```bash
cargo build --release
```

---

# Run Tests

Run the complete Rust workspace test suite:

```bash
cargo test --workspace
```

---

# Project Structure

```text
ProjectDestiny
|
├── crates/
│   |
│   ├── destiny-parser
│   ├── destiny-runtime-core
│   ├── destiny-definition-api
│   ├── destiny-network
│   ├── destiny-server-framework
│   └── destiny-server-runtime
|
├── scripts/
|
├── research tools
|
└── documentation
```

---

# Python Research Tools

Project Destiny includes Python utilities for network research.

## Local DNS Research

Run:

```bash
python3 destiny_local_dns.py
```

## STUN Research

Run:

```bash
python3 stun_redirect.py
```

---

# Generated Files

The following files are generated locally and should not be committed:

```text
target/
*.db
captures/
logs/
__pycache__/
```

These include:

- Rust build artifacts
- Research databases
- Packet captures
- Runtime logs
- Python cache files

---

# Development Workflow

Recommended workflow:

1. Review existing documentation
2. Create a development branch
3. Make changes
4. Run tests
5. Document findings
6. Commit changes

Example:

```bash
git checkout -b feature-name
```

---

# Validation

Before submitting changes:

```bash
cargo test --workspace
cargo build --release
```

---

# Current Development Focus

Active research areas:

- Protocol documentation
- Network analysis
- Runtime framework improvements
- Compatibility research
- Preservation tooling
