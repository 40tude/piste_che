# Quickstart: resort_generator

## Prerequisites

- Rust stable toolchain
- Network access to `overpass-api.de` and `data.geopf.fr`

## Build

```powershell
cargo build -p resort_generator
```

## Run

```powershell
cargo run -p resort_generator -- --resort "Serre Chevalier"
```

## Verify output

```powershell
Get-ChildItem data\serre_chevalier_*.json
```

## Load in main app

```powershell
# Update data file path in config/code if needed, then:
cargo leptos serve
```
