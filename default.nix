{ lib
, stdenv
, craneLib
, fetchYarnDeps
, freetype
, gtk3
, mkYarnPackage
, pkg-config
, rustPlatform
, webkitgtk
, libsoup
, lld
, libayatana-appindicator
, xwin-output
, target ? "linux"
}:

let
	pname = "poe-trade-companion-tauri";
	version = "unstable-2023-10-23";

	frontendFilter = combineFilters [
		(path: _type: builtins.match "^src$" path != null)
		(path: _type: builtins.match ".*json$" path != null)
	];

	src = lib.cleanSourceWith {
		src = ./.;
		filter = frontendFilter;
	};

	frontend-build = mkYarnPackage {
		inherit version;
		pname = "poe-trade-companion-tauri-ui";
		src = ./.;

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

	isLinuxTarget = target == "linux";
	isWindowsTarget = target == "windows";

	tauriConfFilter = path: _type: builtins.match ".*tauri.conf.json$" path != null;
	combineFilters = filters: path: type: builtins.any (x: x path type) filters;
	sourcesFilter = combineFilters [
		tauriConfFilter
		craneLib.filterCargoSources
		(path: _type: builtins.match ".*png$" path != null)
		(path: _type: builtins.match ".*ico$" path != null)
	];

	commonArgs = {
		strictDeps = true;
		doCheck = false;
		src = lib.cleanSourceWith {
			src = craneLib.path ./src-tauri;
			filter = sourcesFilter;
		};
		nativeBuildInputs = [ pkg-config lld ];
		buildInputs = [] ++ lib.optionals isLinuxTarget [ 
			libsoup 
			freetype 
			gtk3 
			webkitgtk 
			libayatana-appindicator
		];
		CARGO_BUILD_TARGET = if isWindowsTarget then "x86_64-pc-windows-msvc" else "x86_64-unknown-linux-gnu";
		CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "lld";
		CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = lib.optional isWindowsTarget ((builtins.foldl' (x: y: x + "-Lnative=${xwin-output}${y} ") "" [
			"/crt/lib/x86_64"
			"/sdk/lib/um/x86_64"
			"/sdk/lib/ucrt/x86_64"
		]) + " -l bufferoverflow");

	};
	cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {});
	ext = lib.optionalString isWindowsTarget ".exe";
in
with craneLib;
buildPackage (commonArgs // {
	inherit cargoArtifacts;
	postPatch = ''
		substituteInPlace tauri.conf.json --replace '"distDir": "../public",' '"distDir": "${frontend-build}",'
	'';
	postInstall = ''
		mv $out/bin/app${builtins.toString ext} $out/bin/poe-trade-companion${builtins.toString ext}
	'';
	cargoBuildCommand = "cargo build --profile release --features custom-protocol";
})