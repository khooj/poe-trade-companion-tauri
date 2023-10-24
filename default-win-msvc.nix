{ lib
, craneLib
, mkYarnPackage
, fetchYarnDeps
# , pkgsCross
# , mingw_w64_pthreads
# , windows
# , stdenv
# , buildPackages
# , targetPackages
# , pthreads
# , cc
, lld
, xwin-output
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
		(path: _type: builtins.match ".*ico$" path != null)
	];

	commonArgs = {
		strictDeps = true;
		doCheck = false;
		src = lib.cleanSourceWith {
			src = craneLib.path ./src-tauri;
			filter = sourcesFilter;
		};
		nativeBuildInputs = [ lld ];
		# buildInputs = [ windows.mingw_w64_pthreads ];
		# depsBuildBuild = with pkgsCross; [ mingwW64.stdenv.cc mingwW64.windows.pthreads ];
		# depsBuildBuild = [ cc ]; 
		CARGO_BUILD_TARGET = "x86_64-pc-windows-msvc";
		CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "lld";
		CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = (builtins.foldl' (x: y: x + "-Lnative=${xwin-output}${y} ") "" [
			"/crt/lib/x86_64"
			"/sdk/lib/um/x86_64"
			"/sdk/lib/ucrt/x86_64"
		]) + " -l bufferoverflow";
	};
	cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {

	});
in
with craneLib;
buildPackage (commonArgs // {
	inherit cargoArtifacts ;
	postPatch = ''
		substituteInPlace tauri.conf.json --replace '"distDir": "../public",' '"distDir": "${frontend-build}",'
	'';
	# postInstall = ''
	# 	mv $out/bin/app $out/bin/poe-trade-companion
	# '';
	cargoBuildCommand = "cargo build --profile release --features custom-protocol";
})