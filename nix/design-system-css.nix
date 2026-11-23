{ pkgs }:
pkgs.writeShellApplication {
  name = "footical-design-system-css";
  runtimeInputs = [
    pkgs.coreutils
    pkgs.jq
  ];
  text = ''
    if [ ! -f Cargo.toml ] || [ ! -d crates/website/style ]; then
      echo "footical-design-system-css: run from the footical repository root" >&2
      exit 1
    fi

    manifest_path="$(
      cargo metadata --format-version 1 "$@" \
        | jq --raw-output '.packages[] | select(.name == "design-system") | .manifest_path'
    )"
    if [ -z "$manifest_path" ]; then
      echo "footical-design-system-css: design-system is not a dependency of this workspace" >&2
      exit 1
    fi

    printf '@import "%s/style/design-system.css";\n' "$(dirname "$manifest_path")" \
      > crates/website/style/design-system.generated.css
  '';
}
