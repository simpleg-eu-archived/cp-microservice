import asyncio
import json
import os

from amqp_api_client_py import amqp_input_api

TEST_TIMEOUT_AFTER_SECONDS_ENV = "TEST_TIMEOUT_AFTER_SECONDS"

REQUEST_AMQP_CONFIG = {
    "queue": {
        "name": "dummy",
    },
    "channel": {
        "publish": {
            "mandatory": False,
            "immediate": False,
            "timeout": None
        }
    }
}

RESPONSE_AMQP_CONFIG = {
    "queue": {
        "name": "",
        "passive": False,
        "durable": False,
        "exclusive": False,
        "auto_delete": True,
        "nowait": False,
        "arguments": {}
    },
    "channel": {
        "consume": {
            "no_ack": False,
            "exclusive": False,
            "arguments": {},
            "consumer_tag": None,
            "timeout": None
        }
    }
}


async def correctly_echo_message():
    request = {
        "header": {
            "action": "dummy:action",
            "token": "abcd1234"
        },
        "payload": {
            "message": "1"
        }
    }

    input_api = amqp_input_api.AmqpInputApi(REQUEST_AMQP_CONFIG, RESPONSE_AMQP_CONFIG)

    timeout_after = int(os.environ.get(TEST_TIMEOUT_AFTER_SECONDS_ENV, 5))

    await asyncio.wait_for(input_api.connect(), timeout_after)

    serialized_result = await asyncio.wait_for(input_api.send_request(request), timeout_after)

    result = json.loads(serialized_result)

    assert ("Ok" in result)
    result = result["Ok"]

    assert ("message" in result)
    assert (request["payload"]["message"] == result["message"])


async def main():
    result_code = 0
    try:
        await correctly_echo_message()
    except BaseException as e:
        print(f"Exception: {e}")
        result_code = 1

    exit(result_code)


if __name__ == "__main__":
    asyncio.run(main())
