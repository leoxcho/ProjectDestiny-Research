# Project Destiny Architecture

## Overview

Project Destiny is built as a modular Rust-based research framework designed to study Destiny 1 client/server architecture, extracted data structures, networking behavior, and experimental preservation infrastructure.

The architecture separates data analysis, runtime services, networking research, and compatibility experiments into independent components.

---

# System Architecture

             Destiny Data Sources
                     |
                     v
          destiny-parser
                     |
                     v
              Definition Index
                     |
                     v
         destiny-runtime-core
                     |
      +--------------+--------------+
      |                             |
      v                             v

destiny-definition-api        server-framework
|                             |
+–––––––+–––––––+
|
v
destiny-server-runtime
|
v
Network Research Layer

---

# Core Components

## destiny-parser

Purpose:

- Parse extracted Destiny definition data
- Identify structures and relationships
- Build searchable metadata indexes

Responsibilities:

- Definition8080 parsing
- Tag extraction
- Reference discovery
- Metadata processing

---

## destiny-runtime-core

Purpose:

Provide shared runtime functionality for accessing preserved research data.

Responsibilities:

- Runtime data management
- Shared interfaces
- Core abstractions

---

## destiny-definition-api

Purpose:

Expose extracted definition information through an API layer.

Responsibilities:

- Query indexed data
- Provide runtime access to definitions
- Support tooling and analysis

---

## destiny-network

Purpose:

Experimental networking research layer.

Current focus:

- Connection observation
- Protocol investigation
- Packet analysis
- Service communication research

---

## destiny-server-framework

Purpose:

Foundation for experimental backend service architecture.

Designed to support:

- Service registration
- Runtime communication
- Modular backend components

---

## destiny-server-runtime

Purpose:

Experimental runtime executable layer.

Current role:

- Service execution testing
- Framework integration
- Future compatibility research

---

# Data Flow

Current research pipeline:

Extracted Data
|
v
Parser
|
v
Indexed Database
|
v
Runtime Services
|
v
Research Tooling

---

# Design Principles

## Modular Development

Each subsystem is isolated to allow independent testing and development.

## Research First

Experiments are documented before implementation decisions are made.

## Reproducibility

Tools and findings are structured so future researchers can repeat analysis.

## Preservation

The goal is documenting technical knowledge and creating open research infrastructure.

---

# Current Development Status

Completed:

- Definition extraction pipeline
- Rust workspace foundation
- Runtime framework
- API layer
- Network research tooling

In Progress:

- Protocol documentation
- Service simulation research
- Compatibility testing


