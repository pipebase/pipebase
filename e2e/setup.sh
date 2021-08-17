#!/usr/bin/env bash

set -o nounset
set -e

# Setup: copy project for app docker build
# Entrypoint: main

project_root="./"
tmp="/tmp/pipebase"
cargo_pipe=""
e2e=""
pipebase=""
pipederive=""
pipegen=""
pipeware=""
cargo_toml=""

function usage() { 
cat <<EOF
setup
Options:
	-d | --directory
	path to pipebase project root directory
    -t | --temporary
    temporary directory
	-h | --help
	print usage
Usage:
	$0 -d </PATH/TO/PIPEBASE>
EOF
exit 1
}

function parse_args() {
	while [[ $# -gt 0 ]]
	do
		i="$1"
		case ${i} in
			-d|--directory)
			if [ $# -lt 2 ]; then
				usage
			fi
			project_root="$2"
			shift
			shift
			;;
            -t|--temporary)
			if [ $# -lt 2 ]; then
				usage
			fi
			tmp="$2"
			shift
			shift
			;;
			-h|--help)
			usage
			shift
			shift
			;;
			*)
			usage
			;;
		esac
	done
	if [ -z "${project_root}" ]; then
		usage
	fi
	if [ ! -d "${project_root}" ]; then
		echo "Project root directory ${project_root} not found, exit ..." 1>&2;
		exit 1;
	fi
}

function init() {
    cargo_pipe="${project_root}/cargo-pipe"
    if [ ! -d ${cargo_pipe} ]; then
        echo "cargo-pipe '${cargo_pipe}' not found"
        exit 1
    fi
    e2e="${project_root}/e2e"
    if [ ! -d ${e2e} ]; then
        echo "e2e '${e2e}' not found"
        exit 1
    fi
    pipebase="${project_root}/pipebase"
    if [ ! -d ${pipebase} ]; then
        echo "pipebase '${pipebase}' not found"
        exit 1
    fi
    pipederive="${project_root}/pipederive"
    if [ ! -d ${pipederive} ]; then
        echo "pipederive '${pipederive}' not found"
        exit 1
    fi
    pipegen="${project_root}/pipegen"
    if [ ! -d ${pipegen} ]; then
        echo "pipegen '${pipegen}' not found"
        exit 1
    fi
    pipeware="${project_root}/pipeware"
    if [ ! -d ${pipeware} ]; then
        echo "pipeware '${pipeware}' not found"
        exit 1
    fi
    cargo_toml="${project_root}/Cargo.toml"
    if [ ! -f ${cargo_toml} ]; then
        echo "Cargo.toml '${cargo_toml}' not found"
        exit 1
    fi
}

function cleanup() {
    rm -r "${e2e}/pipebase"
}

function setup() {
    rm -rf "${e2e}/pipebase"
    rm -rf ${tmp}
    mkdir -p ${tmp}
    cp -r "${cargo_pipe}" "${tmp}/cargo-pipe"
    cp -r "${e2e}" "${tmp}/e2e"
    cp -r "${pipebase}" "${tmp}/pipebase"
    cp -r "${pipederive}" "${tmp}/pipederive"
    cp -r "${pipegen}" "${tmp}/pipegen"
    cp -r "${pipeware}" "${tmp}/pipeware"
    cp "${cargo_toml}" "${tmp}/Cargo.toml"
    mv ${tmp} "${e2e}/pipebase"
}

function main() {
    parse_args $@
    init
    setup
}

main $@