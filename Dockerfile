FROM rust:1.72.0
RUN apt-get update

# Python
RUN apt-get -y install python3-full