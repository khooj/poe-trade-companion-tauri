{ xwin
, stdenv
, lib
}:
stdenv.mkDerivation {
	name = "xwin-output-0.3.1";

	buildInputs = [ xwin ];

	unpackPhase = "true";
	configurePhase = "true";

	buildPhase = ''
		runHook preBuild

		xwin --accept-license splat --output xwin-data

		runHook postBuild
	'';

	installPhase = ''
		runHook preInstall
		mkdir $out
		cp -r ./xwin-data/* $out/
		runHook postInstall
	'';

	impureEnvVars = lib.fetchers.proxyImpureEnvVars;

	outputHashMode = "recursive";
	outputHashAlgo = "sha256";
	outputHash = "sha256-FwI0i9c9pO9jO55MCYGF87nFFRxgQSWu+aSosUctoIE=";
}
