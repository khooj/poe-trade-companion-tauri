{ lib
, craneLib
, fetchYarnDeps
, freetype
, gtk3
, mkYarnPackage
, pkg-config
, rustPlatform
, webkitgtk
, libsoup
}:

let
	pname = "poe-trade-companion-tauri";
	version = "unstable-2023-10-23";

	src = ./.;

	frontend-build = mkYarnPackage {
		inherit version src;
		pname = "poe-trade-companion-tauri-ui";

		offlineCache = fetchYarnDeps {
			yarnLock = src + "/yarn.lock";
			sha256 = "sha256-lT2Ny11ABe5nou6OauZ1UjI959PlJZxMoxPRvZMjFRY=";
		};

		packageJSON = ./package.json;

		buildPhase = ''
			export HOME=$(mktemp -d)
			yarn --offline run build
			cp -r deps/poe-trade-companion-tauri/build $out
		'';

		distPhase = "true";
		dontInstall = true;
	};

# rustPlatform.buildRustPackage {
# 	inherit version src pname;

# 	sourceRoot = "${src}/src-tauri";

# 	dontMakeSourcesWritable = true;

# 	postUnpack = ''
# 	'';

# 	unpackPhase = "true";

# 	cargoLock = {
# 		lockFile = ./src-tauri/Cargo.lock;
# 		outputHashes = {
# 			"tauri-plugin-log-0.0.0" = "sha256-AQt2cJ3ZP0Ffme2lfThcjSUA6FDE6srJryaNeMRXpz0=";
# 		};
# 	};

# 	postPatch = ''
# 		cp ${./src-tauri/Cargo.lock} Cargo.lock
# 		mkdir -p frontend-build
# 		cp -R ${frontend-build} frontend-build
# 		# ls -la .
# 		substituteInPlace tauri.conf.json --replace '"distDir": "../public",' '"distDir": "frontend-build",'
# 	'';

# 	buildInputs = [ gtk3 webkitgtk freetype ];
# 	nativeBuildInputs = [ pkg-config ];

# 	checkFlags = [];

# 	postInstall = ''
# 		mv $out/bin/app $out/bin/poe-trade-companion
# 	'';
# }

	tauriConfFilter = path: _type: builtins.match ".*tauri.conf.json$" path != null;
	# sourcesFilter = path: type: (tauriConfFilter path type) || (craneLib.filterCargoSources path type);
	filtersCombinator = filters: path: type: builtins.any (x: x path type) filters;
	sourcesFilter = filtersCombinator [
		tauriConfFilter
		craneLib.filterCargoSources
		(path: _type: builtins.match ".*png$" path != null)
	];

	commonArgs = {
		src = lib.cleanSourceWith {
			src = craneLib.path ./src-tauri;
			filter = sourcesFilter;
		};
		nativeBuildInputs = [ pkg-config ];
		buildInputs = [ libsoup freetype gtk3 webkitgtk ];
	};
	cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {

	});
in
with craneLib;
buildPackage (commonArgs // {
	inherit cargoArtifacts;
	postPatch = ''
		substituteInPlace tauri.conf.json --replace '"distDir": "../public",' '"distDir": "${frontend-build}",'
	'';
	postInstall = ''
		mv $out/bin/app $out/bin/poe-trade-companion
	'';
	cargoBuildCommand = "cargo build --profile release --features custom-protocol";
})