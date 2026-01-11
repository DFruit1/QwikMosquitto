{ lib, rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "mqtt-recorder";
  version = "0.1.0";

  src = ../../backend;

  cargoLock = {
    lockFile = ../../backend/Cargo.lock;
  };

  meta = with lib; {
    description = "Records MQTT messages into Postgres";
    license = licenses.mit;
    mainProgram = "mqtt-recorder";
  };
}
