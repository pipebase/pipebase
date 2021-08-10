#!/usr/bin/env bash
set -o nounset
set -o pipefail

# Checker: validate and check pipe manifest
# Entrypoint: main

# checklist json file
checklist_file=""

function log() {
	echo -e "[$(date -u +"%Y-%m-%d %H:%M:%S UTC")] $@"
}

function debug() {
	log "[DEBUG] [$@]"
}

function info() {
	log "[INFO] [$@]"
}

function error() {
	log "[ERROR] [$@]"
}

function usage() { 
cat <<EOF
Checker
Options:
	-f | --file
	path to example checklist (required)
	-h | --help
	print usage
Usage:
	$0 -f </PATH/TO/CHECKLIST>
EOF
exit 1
}

function parse_args() {
	while [[ $# -gt 0 ]]
	do
		i="$1"
		case ${i} in
			-f|--file)
			if [ $# -lt 2 ]; then
				usage
			fi
			checklist_file="$2"
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
	if [ -z "${checklist_file}" ]; then
		usage
	fi
	if [ ! -f "${checklist_file}" ]; then
		echo "File ${checklist_file} not found, exit ..." 1>&2;
		exit 1;
	fi
}

function do_remove() {
    local directory=${1}
    local manifest=${2}
    cargo pipe -d ${directory} -m ${manifest} remove
}

function do_create() {
    local directory=${1}
    local manifest=${2}
    cargo pipe -d ${directory} -m ${manifest} new
    cargo pipe -d ${directory} -m ${manifest} generate
}

function do_check() {
    local directory=${1}
    local manifest=${2}
    cargo pipe -d ${directory} -m ${manifest} check -d
}

function do_validate() {
    local directory=${1}
    local manifest=${2}
    cargo pipe -d ${directory} -m ${manifest} validate -p -o -c
}

function check() {
    local app_idx=${1}
    local app_name=$(jq -r ".[${app_idx}].name" "${checklist_file}")
    local directory=$(jq -r ".[${app_idx}].directory" "${checklist_file}")
    local action="check"
    local manifest="pipe.yml"
    local is_manifest_defined=$(jq -r ".[${app_idx}] | has(\"manifest\")" "${checklist_file}")
    if [ "${is_manifest_defined}" = "true" ]; then
        manifest=$(jq -r ".[${app_idx}].manifest" "${checklist_file}")
    fi
    local is_action_defined=$(jq -r ".[${app_idx}] | has(\"action\")" "${checklist_file}")
    if [ "${is_action_defined}" = "true" ]; then
        action=$(jq -r ".[${app_idx}].action" "${checklist_file}")
    fi
    if [ ! -d "${directory}" ]; then
        error "directory ${directory} not exists"
        exit 1
    fi
    if [ ! -f "${directory}/${manifest}" ]; then
        error "manifest ${manifest} not exists in ${directory}"
        exit 1
    fi
    if [ "${action}" = "validate" ]; then
        do_validate ${directory} ${manifest}
        local exit_code=$?
        if [ $exit_code -ne 0 ]; then
            error "validate ${app_name} failed"
            return 1        
        fi
        return 0
    fi
    do_remove ${directory} ${manifest}
    local removed=$?
    if [ $removed -ne 0 ]; then
        error "failed to cleanup app ${app_name}"
        return 1
    fi
    do_create ${directory} ${manifest}
    local created=$?
    if [ $created -ne 0 ]; then
        error "failed to create app ${app_name}"
        return 1
    fi
    if [ "${action}" = "check" ]; then
        do_check ${directory} ${manifest}
        local exit_code=$?
        if [ $exit_code -ne 0 ]; then
            error "check ${app_name} failed"
            return 1        
        fi
        return 0
    fi
    error "Undefined action ${action} for ${app_name} at entry ${app_idx}"
    return 1
}

function start_checker() {
    info "Checker start ..."
    local is_checklist_array=$(jq ". | if type==\"array\" then \"true\" else \"false\" end" "${checklist_file}")
    if [ "${is_checklist_array}" = "false" ]; then
		error "checklist should be array type"
		return 1
	fi
    local app_count=$(jq ". | length" "${checklist_file}")
    local app_idx=0
    # pass flag, all pass - 0, fail any - 1
    local pass=0
    while [ ${app_idx} -lt ${app_count} ]; do
        check ${app_idx}
        local return_code=$?
        if [ ${return_code} -ne 0 ]; then
            pass=1
        fi 
        app_idx=$((app_idx + 1))
    done
    return ${pass} 
}

# validate whether file in correct json format
function validate_json_format() {
    echo "$1"
    if [ ! -f "$1" ]; then
        error "File ${1} not exists."
        return 1
    fi
    jq "." "$1"
    if [ $? -ne 0 ]; then
        error "File ${1} with wrong format."
        return 1
    fi
    return 0
}

function init_checker() {
    info "Initialize checker ..."
    validate_json_format "${checklist_file}"
	if [ $? -ne 0 ]; then
		error "Corrupted checklist file ${checklist_file}"
		exit 1
	fi
}

# Entrypoint of checker
function main() {
    parse_args $@
    init_checker
    start_checker
    exit $?
}

main $@