{ config, lib, pkgs, ... }:
let
  cfg = config.services.mqttRecorder;
  defaultPackage = pkgs.callPackage ../packages/mqtt-recorder.nix { };
in
{
  options.services.mqttRecorder = {
    enable = lib.mkEnableOption "MQTT recorder service";

    package = lib.mkOption {
      type = lib.types.package;
      default = defaultPackage;
      description = "Package providing the mqtt-recorder binary.";
    };

    databaseUrl = lib.mkOption {
      type = lib.types.str;
      default = "postgres://postgres:postgres@localhost:5432/mqtt";
      description = "Postgres connection URL.";
    };

    mqttHost = lib.mkOption {
      type = lib.types.str;
      default = "localhost";
      description = "MQTT broker host.";
    };

    mqttPort = lib.mkOption {
      type = lib.types.port;
      default = 1883;
      description = "MQTT broker port.";
    };

    mqttClientId = lib.mkOption {
      type = lib.types.str;
      default = "mqtt-recorder";
      description = "MQTT client identifier.";
    };

    mqttTopic = lib.mkOption {
      type = lib.types.str;
      default = "#";
      description = "Topic filter to subscribe to.";
    };

    httpHost = lib.mkOption {
      type = lib.types.str;
      default = "0.0.0.0";
      description = "Host/interface for the HTTP API.";
    };

    httpPort = lib.mkOption {
      type = lib.types.port;
      default = 8080;
      description = "Port for the HTTP API.";
    };

    user = lib.mkOption {
      type = lib.types.str;
      default = "mqtt-recorder";
      description = "User account for the recorder service.";
    };

    group = lib.mkOption {
      type = lib.types.str;
      default = "mqtt-recorder";
      description = "Group for the recorder service.";
    };
  };

  config = lib.mkIf cfg.enable {
    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
    };

    users.groups.${cfg.group} = { };

    systemd.services.mqtt-recorder = {
      description = "MQTT message recorder";
      wantedBy = [ "multi-user.target" ];
      after = [ "network-online.target" "mosquitto.service" "postgresql.service" ];
      requires = [ "mosquitto.service" "postgresql.service" ];
      serviceConfig = {
        User = cfg.user;
        Group = cfg.group;
        ExecStart = "${cfg.package}/bin/mqtt-recorder";
        Restart = "on-failure";
        Environment = [
          "DATABASE_URL=${cfg.databaseUrl}"
          "MQTT_HOST=${cfg.mqttHost}"
          "MQTT_PORT=${toString cfg.mqttPort}"
          "MQTT_CLIENT_ID=${cfg.mqttClientId}"
          "MQTT_TOPIC=${cfg.mqttTopic}"
          "APP_HOST=${cfg.httpHost}"
          "APP_PORT=${toString cfg.httpPort}"
        ];
      };
    };
  };
}
