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

result_exit_code=0

# TEST EXIT, EXPECTED EXIT CODE: 1
./test_exit

test_exit_code=$?
if [ $test_exit_code -eq 1 ]; then
  echo "Test exit: SUCCESS"
else
  echo "Test exit: FAILED"
  result_exit_code=1
fi

# TEST RUN, EXPECTED EXIT CODE: 0
./test_run

test_run_code=$?
if [ $test_run_code -eq 0 ]; then
  echo "Test run: SUCCESS"
else
  echo "Test run: FAILED"
  result_exit_code=1
fi

# TEST WAIT FOR TASKS TO END, EXPECTED EXIT CODE: 1
./test_wait_for_tasks_to_end

test_wait_for_tasks_to_end_code=$?
if [ $test_wait_for_tasks_to_end_code -eq 1 ]; then
  echo "Test wait for tasks to end: SUCCESS"
else
  echo "Test wait for tasks to end: FAILED"
  result_exit_code=1
fi

# INTEGRATION TESTS

cd ../../

# TEST AMQP API IMPL, EXPECTED EXIT CODE: 0
./target/debug/test_amqp_api_impl &
impl_pid=$!

sleep 1
python3 integration_tests/test_amqp_api_impl.py

test_amqp_api_impl_code=$?
if [ $test_amqp_api_impl_code -eq 0 ]; then
  echo "Test AMQP API impl: SUCCESS"
else
  echo "Test AMQP API impl: FAILED"
  result_exit_code=1
fi

kill $impl_pid
exit $result_exit_code