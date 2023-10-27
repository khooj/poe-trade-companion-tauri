{ lib
, stdenv
, craneLib
, freetype
, gtk3
, pkg-config
, rustPlatform
, webkitgtk
, libsoup
, lld
, libayatana-appindicator
, xwin-output
, poe-trade-companion-ui
, target ? "linux"
}:

let
	isLinuxTarget = target == "linux";
	isWindowsTarget = target == "windows";
	frontend-build = poe-trade-companion-ui;

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