#!/usr/bin/env bash

# Validator: run tests
# Entrypoint: main

set -o nounset
set -o pipefail

# run date - 
run_date=$(date -u +"%Y-%m-%d-%H:%M:%S-UTC")
# dump file prefix
dump_file_prefix="TEST-${run_date}"
# test index file
index_file=""

# start line space of test stats
function incr_test_stats_line_start_space() {
	stats_line_space_level=$((stats_line_space_level + 1))
}

function decr_test_stats_line_start_space() {
	stats_line_space_level=$((stats_line_space_level - 1))
}

# get timestamp in millisecond
function timestamp() {
	echo $(date +"%s%3")
}

function get_duration() {
    local start=$1
	local now=$(timestamp)
    local duration=0
    if [[ $now -gt $start ]]; then
        duration=$((now - start))
    fi
	echo $duration
}

function create_dir() {
	if [ ! -d "${1}" ]; then
		mkdir -p "${1}"
		if [ $? -ne 0 ]; then
			error "Failed to create directory ${1}"
			return 1
		fi
	fi
	return 0
}

# return 0 - diff directory created, 1 - diff directory undefined
function init_diff_dir() {
	is_diff_dir_defined=$(jq -r ". | has(\"diff_dir\")" "${index_file}")
	if [ "${is_diff_dir_defined}" = "false" ]; then
		info "Diff directory is required, exit ..."
		exit 1
	fi
	diff_dir=$(jq -r '.diff_dir' "${index_file}")
	create_dir "${diff_dir}"
	# tmp file for output diff purpose
	tmp_out_file="${diff_dir}/tmp.out"
}

function clean_tmp_file() {
	if [ -f "${tmp_out_file}" ]; then
		rm "${tmp_out_file}"
	fi
}

# return 0 - stats directory created, 1 - stats directory undefined
function init_stats_dir() {
	is_stats_dir_defined=$(jq -r ". | has(\"stats_dir\")" "${index_file}")
	if [ "${is_stats_dir_defined}" = "false" ]; then
		info "Non stats directory defined, no tests stats will be dumped ..."
		return 1
	fi
	stats_dir=$(jq -r '.stats_dir' "${index_file}")
	create_dir "${stats_dir}"
	stats_file="${stats_dir}/${dump_file_prefix}.stats"
	# test stats line space level
	stats_line_space_level=0
}

function init_log_dir() {
	is_log_dir_defined=$(jq -r ". | has(\"log_dir\")" "${index_file}")
	if [ "${is_log_dir_defined}" = "false" ]; then
		info "Non log directory defined, no log will be dumped ..."
		return 1
	fi
	log_dir=$(jq -r '.log_dir' "${index_file}")
	create_dir "${log_dir}"
	log_file="${log_dir}/${dump_file_prefix}.log"
	return 0
}

# dump test stats header
function dump_test_stats_header() {
	if [ "${is_stats_dir_defined}" = "false" ] || [ ! -d "${stats_dir}" ]; then
		return 0
	fi
	for ((space_count=0; space_count < stats_line_space_level; space_count++)); do 
		printf "   " >> "${stats_file}"; 
	done
	echo "$@" >> "${stats_file}"
}

# dump test stats
function dump_test_stats() {
	if [ "${is_stats_dir_defined}" = "false" ] || [ ! -d "${stats_dir}" ]; then
		return 0
	fi
	for ((space_count=0; space_count < stats_line_space_level; space_count++)); do 
		printf "   " >> "${stats_file}"; 
	done
	printf "${1}: #total(${2}), #pass(${3}), #fail(${4}), #ignore(${5}), duration(${6} millis)\n" >> "${stats_file}"
}

# dump one test pass
function dump_test_pass_stats() {
	dump_test_stats ${1} 1 1 0 0 ${2}
}

# dump one test fail
function dump_test_fail_stats() {
	dump_test_stats $1 1 0 1 0 ${2}
}

function log() {
	echo -e "[$(date -u +"%Y-%m-%d %H:%M:%S UTC")] $@"
	if [ "${validator_inited}" = "true" ] && [ "${is_log_dir_defined}" == "true" ] && [ -d "${log_dir}" ]; then
		echo -e "[$(date -u +"%Y-%m-%d %H:%M:%S UTC")] $@" >> "${log_file}"
	fi
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
validator
Options:
	-f | --file
	path to test entry file (required)
	-h | --help
	print usage
Usage:
	$0 -f </PATH/TO/TEST/INDEX>
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
			index_file="$2"
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
	if [ -z "${index_file}" ]; then
		usage
	fi
	if [ ! -f "${index_file}" ]; then
		echo "File ${index_file} not found, exit ..." 1>&2;
		exit 1;
	fi
}

function is_number() {
    re='^[0-9]+$'
    if ! [[ $1 =~ $re ]] ; then
        return 1
    fi
    return 0
}

function run_cmd() {
	eval "$@" > "${tmp_out_file}"
}

# return 0 - pass, 1 - fail
function run_test() {
	clean_tmp_file
	local suite_name="$1"
	local suite_file="$2"
	local test_idx=$3
	# default value for each field
	local cmd=""
	local expected_output_file=""
	local expected_exit_code=0
	local is_cmd_defined=$(jq -r ".[${test_idx}] | has(\"cmd\")" "${suite_file}")
	local is_output_file_defined=$(jq -r ".[${test_idx}] | has(\"expected_output_file\")" "${suite_file}")
	local is_exit_code_defined=$(jq -r ".[${test_idx}] | has(\"expected_exit_code\")" "${suite_file}")
	# check whether field defined and assign value
	if [ "${is_cmd_defined}" = "true" ]; then
		cmd=$(jq -r ".[${test_idx}] | .cmd" "${suite_file}")
	fi
	if [ "${is_output_file_defined}" = "true" ]; then
		expected_output_file=$(jq -r ".[${test_idx}] | .expected_output_file" "${suite_file}")
	fi
	if [ "${is_exit_code_defined}" = "true" ]; then
		expected_exit_code=$(jq -r ".[${test_idx}] | .expected_exit_code" "${suite_file}")
        is_number $expected_exit_code
        local is_exit_code_number=$?
        if [ $is_exit_code_number -eq 1 ]; then
            echo "expected_exit_code($expected_exit_code) not a number"
            return 1
        fi
    fi 
	info "${suite_name}-${test_idx}: '${cmd}' started"
	local start_time=$(timestamp)
	run_cmd $cmd
	local actual_exit_code=$?
	# If we don't care the command success or not just return 
	if [ ${expected_exit_code} -eq -1 ]; then
		info "${suite_name}-${test_idx} pass"
		dump_test_pass_stats "${suite_name}-${test_idx}" $(get_duration ${start_time})
		return 0
	fi
	local out_diff_file="${diff_dir}/${dump_file_prefix}-${suite_name}-${test_idx}.diff"
	# Test failed
	if [ ${actual_exit_code} -ne ${expected_exit_code} ]; then
		local error_message="return code expected ${expected_exit_code} != actual ${actual_exit_code}"
		echo "${error_message}" > "${out_diff_file}"
		error "${suite_name}-${test_idx} fail, ${error_message}, see ${out_diff_file}"
		dump_test_fail_stats "${suite_name}-${test_idx}" $(get_duration ${start_time})
		return 1
	fi
	# Skip output comparison if empty
	if [ -z "${expected_output_file}" ]; then
		info "${suite_name}-${test_idx} pass"
		dump_test_pass_stats "${suite_name}-${test_idx}" $(get_duration ${start_time})
		return 0
	fi
	if [ ! -f "${expected_output_file}" ]; then
		error "${suite_name}-${test_idx} fail, ${expected_output_file} does not exists"
		dump_test_fail_stats "${suite_name}-${test_idx}" $(get_duration ${start_time})
		return 1
	fi
	# diff output
	diff "${tmp_out_file}" "${expected_output_file}" > "${out_diff_file}"
	# dump test failure if non-empty diff
	if [ -f "${out_diff_file}" ] && [ -s "${out_diff_file}" ]; then 
		error "${suite_name}-${test_idx} fail. Actual output different from expected. See ${out_diff_file}"
		dump_test_fail_stats "${suite_name}-${test_idx}" $(get_duration ${start_time})
		return 1
	fi
	# clean diff if pass
	rm "${out_diff_file}" || true
	info "${suite_name}-${test_idx} pass"
	dump_test_pass_stats "${suite_name}-${test_idx}" $(get_duration ${start_time})
	return 0
}

function run_test_retry() {
    local suite_name="$1"
	local suite_file="$2"
	local test_idx=$3
    local retry=0
    local is_retry_defined=$(jq -r ".[${test_idx}] | has(\"retry\")" "${suite_file}")
    if [ "${is_retry_defined}" = "true" ]; then
        retry=$(jq -r ".[${test_idx}] | .retry" "${suite_file}")
        is_number $retry
        local is_retry_number=$?
        if [ $is_retry_number -eq 1 ]; then
            echo "retry($retry) not a number"
            return 1
        fi
    fi
    local try=$(( retry + 1 ))
    local return_code=0
    while [ $try -gt 0 ]; do
        run_test "${suite_name}" "${suite_file}" ${test_idx}
        return_code=$?
        if [ $return_code -eq 0 ]; then
            return 0
        fi
        try=$(( try - 1 ))
        if [ $try -gt 0 ]; then
            sleep 3
        fi
    done
    # all retry failed
    return 1
}

# return 0 - pass, 1 - fail, 2 - ignored
function run_suite() {
	local suite_idx=${1}
	local suite_name=$(jq -r ".suites[${suite_idx}].name" "${index_file}")
	# ignored flag defined ?
	local is_ignored_defined=$(jq -r ".suites[${suite_idx}] | has(\"is_ignored\")" "${index_file}")
	if [ "${is_ignored_defined}" = "true" ]; then
		# ignored ?
		local is_ignored=$(jq -r ".suites[${suite_idx}].is_ignored" "${index_file}")
		if [ ! -z "${is_ignored}" ] && [ "${is_ignored}" = "true" ]; then
			info info "suite-${suite_idx}: ${suite_name} ignored ..."
			dump_test_stats_header "suite-${suite_idx}: ${suite_name} ignored"
			return 2
		fi
	fi
	# check whether suite file defined
	local is_suite_file_defined=$(jq -r ".suites[${suite_idx}] | has(\"file\")" "${index_file}")
	if [ "${is_suite_file_defined}" = "false" ]; then
		error "test suite file not defined for ${suite_name}"
		return 1
	fi
	# find tests in suite
	local suite_file=$(jq -r ".suites[${suite_idx}].file" "${index_file}")
	info "suite-${suite_idx}: ${suite_name} ${suite_file} start ..."
	validate_json_format "${suite_file}"
	if [ $? -ne 0 ]; then
		error "Corrupted suite file ${suite_file} ..."
		return 1
	fi
	# check suite is an array of tests
	local is_suite_array_type=$(jq ". | if type==\"array\" then \"true\" else \"false\" end" "${suite_file}")
	if [ "${is_suite_array_type}" = "false" ]; then
		error "suite should be array type ${suite_file}"
		return 1
	fi
	local test_count=$(jq ". | length" "${suite_file}")
	local test_idx=0
	local pass_count=0
	local fail_count=0
	local pass=0
	local return_code=0
	local start_time=$(timestamp)
	dump_test_stats_header "suite-${suite_idx}: ${suite_file}"
	incr_test_stats_line_start_space
	while [ ${test_idx} -lt ${test_count} ]; do
		run_test_retry "${suite_name}" "${suite_file}" ${test_idx}
		return_code=$?
		# collect test stats
		if [ ${return_code} -eq 0 ]; then
			pass_count=$((pass_count + 1))
		elif [ ${return_code} -eq 1 ]; then
			pass=1
			fail_count=$((fail_count + 1))
		else
			debug "Unexpected return code(${return_code}) from run_test"
		fi
		test_idx=$((test_idx + 1))
	done
	decr_test_stats_line_start_space
	dump_test_stats "suite-${suite_idx}" ${test_count} ${pass_count} ${fail_count} 0 $(get_duration ${start_time})
	return ${pass}
}

# run test suites
# return 0 - all suites pass, 1 - fail
function start_validator() {
	info "Validator ${validator_name} start ..."
	local start_time=$(timestamp)
	local is_suites_defined=$(jq ". | has(\"suites\")" "${index_file}")
	if [ "${is_suites_defined}" = "false" ]; then
		error "suites undefined in ${index_file}"
		return 1
	fi
	# check suites is an array of suite
	local is_suites_array_type=$(jq ".suites | if type==\"array\" then \"true\" else \"false\" end" "${index_file}")
	if [ "${is_suites_array_type}" = "false" ]; then
		error "suites should be array type in ${index_file}"
		return 1
	fi
	local suite_count=$(jq ".suites | length" "${index_file}")
	local suite_idx=0
	local pass_count=0
	local fail_count=0
	local ignore_count=0
	local pass=0
	local return_code=0
	dump_test_stats_header "${validator_name}: ${index_file}"
	incr_test_stats_line_start_space
	while [ ${suite_idx} -lt ${suite_count} ]; do
		run_suite ${suite_idx}
		return_code=$?
		# collect suite stats
		if [ ${return_code} -eq 0 ]; then
			pass_count=$((pass_count + 1))
		elif [ ${return_code} -eq 1 ]; then
			pass=1
			fail_count=$((fail_count + 1))
		elif [ ${return_code} -eq 2 ]; then
			ignore_count=$((ignore_count + 1))
		else
			error "Unexpected return value(${return_code}) from run_suite"
		fi
		suite_idx=$((suite_idx + 1))
	done
	decr_test_stats_line_start_space
	dump_test_stats "${validator_name}" ${suite_count} ${pass_count} ${fail_count} ${ignore_count} $(get_duration ${start_time})
	info "Validator ${validator_name} done ..."
	if [ ${pass} -eq 0 ]; then
		info "Pass!"
	else
		info "Failure!"
	fi
	# print stats/log file
	if [ "${is_stats_dir_defined}" = "true" ] && [ -f "${stats_file}" ]; then
		info "Stats dump: ${stats_file}"
		cat "${stats_file}"
	fi
	if [ "${is_log_dir_defined}" = "true" ] && [ -f "${log_file}" ]; then
		info "Log dump: ${log_file}"
	fi
	# print diff if any
	for diff_file in ${diff_dir}/*.diff; do
		if [ -f $diff_file ]; then
			info "Diff dump: ${diff_file}"
			cat $diff_file
		fi
	done
	clean_tmp_file
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

# 1. Read test index file
# 2. Init diff/stats/log directory
function init_validator() {
	validator_inited="false"
	info "Initialize test environment ..."
	validate_json_format "${index_file}"
	if [ $? -ne 0 ]; then
		error "Corrupted test entry file ${index_file}"
		exit 1
	fi
	
	validator_name=$(jq -r ".name" "${index_file}")
	dump_file_prefix="${dump_file_prefix}-${validator_name}"
	init_diff_dir
	init_stats_dir
	init_log_dir
	validator_inited="true"
}

# Entrypoint of validator
function main() {
	parse_args $@
	init_validator
	start_validator
	exit $?
}

main $@