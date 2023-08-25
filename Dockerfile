FROM rust:1.72.0
RUN apt-get update

# Python
RUN apt-get -y install python3 python3-pip python3-venv
RUN pip3 install amqp_api_client_py aio_pika