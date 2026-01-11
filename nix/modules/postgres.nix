{ nixhomeserver, ... }:
{
  imports = [ nixhomeserver.nixosModules.postgres ];
}
