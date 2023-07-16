#!/bin/bash

cargo build

build_code=$?

if [ $build_code -ne 0 ]; then
  echo "Build: FAILED"
  exit 1
else
  echo "Build: SUCCESS"
fi

cd ./target/debug

resultExitCode=0

# TEST EXIT, EXPECTED EXIT CODE: 1
./test_exit

test_exit_code=$?
if [ $test_exit_code -eq 1 ]; then
  echo "Test exit: SUCCESS"
else
  echo "Test exit: FAILED"
  resultExitCode=1
fi

# TEST RUN, EXPECTED EXIT CODE: 0
./test_run

test_run_code=$?
if [ $test_run_code -eq 0 ]; then
  echo "Test run: SUCCESS"
else
  echo "Test run: FAILED"
  resultExitCode=1
fi

# TEST WAIT FOR TASKS TO END, EXPECTED EXIT CODE: 1
./test_wait_for_tasks_to_end

test_wait_for_tasks_to_end_code=$?
if [ $test_wait_for_tasks_to_end_code -eq 1 ]; then
  echo "Test wait for tasks to end: SUCCESS"
else
  echo "Test wait for tasks to end: FAILED"
  resultExitCode=1
fi

exit $resultExitCode
