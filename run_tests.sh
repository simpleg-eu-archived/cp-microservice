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

# TEST PROCESS SIGNAL SIGINT HANDLING, EXPECTED EXIT CODE: 0

./test_process_signal_sigint_handling

test_process_signal_sigint_handling_code=$?

if [ $test_process_signal_sigint_handling_code -eq 0 ]; then
  echo "Test process signal SIGINT handling: SUCCESS"
else
  echo "Test process signal SIGINT handling: FAILED"
  result_exit_code=1
fi

# TEST PROCESS SIGNAL SIGTERM HANDLING, EXPECTED EXIT CODE: 0

./test_process_signal_sigterm_handling

test_process_signal_sigterm_handling_code=$?

if [ $test_process_signal_sigterm_handling_code -eq 0 ]; then
  echo "Test process signal SIGTERM handling: SUCCESS"
else
  echo "Test process signal SIGTERM handling: FAILED"
  result_exit_code=1
fi

# TEST PROCESS SIGNAL SIGQUIT HANDLING, EXPECTED EXIT CODE: 0

./test_process_signal_sigquit_handling

test_process_signal_sigquit_handling_code=$?

if [ $test_process_signal_sigquit_handling_code -eq 0 ]; then
  echo "Test process signal SIGQUIT handling: SUCCESS"
else
  echo "Test process signal SIGQUIT handling: FAILED"
  result_exit_code=1
fi

# TEST AMQP API IMPL, EXPECTED EXIT CODE: 0

DEFAULT_AMQP_CONNECTION_URI="amqp://guest:guest@127.0.0.1:5672"
TEST_AMQP_CONNECTION_URI=${TEST_AMQP_CONNECTION_URI:=$DEFAULT_AMQP_CONNECTION_URI}

./test_amqp_api_impl_server $TEST_AMQP_CONNECTION_URI &
impl_pid=$!

sleep 1
./test_amqp_api_impl_client $TEST_AMQP_CONNECTION_URI

test_amqp_api_impl_code=$?
if [ $test_amqp_api_impl_code -eq 0 ]; then
  echo "Test AMQP API impl: SUCCESS"
else
  echo "Test AMQP API impl: FAILED"
  result_exit_code=1
fi

kill $impl_pid
exit $result_exit_code