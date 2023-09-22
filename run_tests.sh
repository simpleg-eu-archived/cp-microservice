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

# TEST AMQP API IMPL, EXPECTED EXIT CODE: 0

DEFAULT_AMQP_CONNECTION_URI="amqp://guest:guest@127.0.0.1:5672"
TEST_AMQP_CONNECTION_URI=${TEST_AMQP_CONNECTION_URI:=$DEFAULT_AMQP_CONNECTION_URI}

./target/debug/test_amqp_api_impl_server $TEST_AMQP_CONNECTION_URI &
impl_pid=$!

sleep 1
./target/debug/test_amqp_api_impl_client $TEST_AMQP_CONNECTION_URI

test_amqp_api_impl_code=$?
if [ $test_amqp_api_impl_code -eq 0 ]; then
  echo "Test AMQP API impl: SUCCESS"
else
  echo "Test AMQP API impl: FAILED"
  result_exit_code=1
fi

kill $impl_pid
exit $result_exit_code