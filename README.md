# Qwik Mosquitto Recorder

This repository contains a starter stack for building a Qwik frontend and a Rust backend that records MQTT messages into Postgres and exposes them over an HTTP API.

## Layout

- `backend/` - Rust MQTT recorder service with Axum API.
- `web/` - Qwik City UI scaffold (Vite-powered).
- `nix/` - NixOS modules and package definitions.

## Rust MQTT recorder + API

The recorder subscribes to an MQTT topic, inserts incoming payloads into Postgres, and exposes the data through an Axum API.

```bash
cd backend
cp .env.example .env
cargo run
```

Environment variables:

- `DATABASE_URL`
- `MQTT_HOST`
- `MQTT_PORT`
- `MQTT_CLIENT_ID`
- `MQTT_TOPIC`
- `APP_HOST`
- `APP_PORT`

HTTP endpoints:

- `GET /health`
- `GET /messages?limit=50`

## Qwik UI

```bash
cd web
npm install
npm run dev
```

## Nix

The flake provides a development shell, a package for the Rust service, and NixOS modules.

```bash
nix develop
```

### NixOS modules pinned to nixhomeserver

The Mosquitto and Postgres modules are imported from the `nixhomeserver` flake input so you can pin service versions to match that repository. Update the flake input to point at your homeserver once it is available.

Example:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    nixhomeserver.url = "github:your-org/nixhomeserver";
    qwik-mosquitto.url = "path:/path/to/QwikMosquitto";
  };

  outputs = { self, nixpkgs, nixhomeserver, qwik-mosquitto }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in {
      nixosConfigurations.dev = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = { inherit nixhomeserver; };
        modules = [
          qwik-mosquitto.nixosModules.mosquitto
          qwik-mosquitto.nixosModules.postgres
          qwik-mosquitto.nixosModules.mqtt-recorder
          ({
            services.mqttRecorder.enable = true;
            services.mqttRecorder.databaseUrl = "postgres://postgres:postgres@localhost:5432/mqtt";
            services.mqttRecorder.httpPort = 8080;
          })
        ];
      };
    };
}
```
