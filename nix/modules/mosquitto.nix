{ nixhomeserver, ... }:
{
  imports = [ nixhomeserver.nixosModules.mosquitto ];
}
