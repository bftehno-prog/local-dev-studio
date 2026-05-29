# Architecture

## Overview

Local Dev Studio combines a React frontend, a Tauri Rust backend and a SQLite database.

## Frontend

The frontend uses React and TypeScript. The shell manages navigation and feature views. Shared UI primitives live in `src/components/ui`, while routes and defaults are split into `src/app` and `src/lib`.

## Backend

The backend exposes Tauri commands for projects, servers, ports, logs, settings, templates and diagnostics. It owns command construction and process lifecycle so the UI never executes arbitrary shell commands.

## SQLite

SQLite stores projects, settings, process records, logs, templates and sandboxes. Schema changes are tracked with `schema_migrations`.

## Process Manager

Processes are started from allow-listed project types. Startup is non-blocking; a background monitor updates `starting`, `running` and `error` status.

## Preview System

Preview uses local URLs, iframe rendering, external browser launch and optional LAN URLs for QR codes.

## Security Model

The backend validates project paths, project types, package managers, ZIP paths and environment variable format before launch or import.
