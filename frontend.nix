{ mkYarnPackage
, fetchYarnDeps
, lib
}:
let
	src = lib.sourceFilesBySuffices ./. [
		".js" ".svelte" ".json" ".css" ".html" ".png"
		".cjs" "yarn.lock"
	];
in
mkYarnPackage {
	version =  "unstable-2023-10-27";
	pname = "poe-trade-companion-tauri-ui";
	inherit src;

	offlineCache = fetchYarnDeps {
		yarnLock = src + "/yarn.lock";
		sha256 = "sha256-yPbsjfYpQayVw9VRDjUtODq8TbEqxgvpJ63dCjj5BeE=";
	};

	packageJSON = ./package.json;

	buildPhase = ''
		export HOME=$(mktemp -d)
		yarn --offline run build
		cp -r deps/poe-trade-companion-tauri/build $out
	'';

	distPhase = "true";
	dontInstall = true;
}