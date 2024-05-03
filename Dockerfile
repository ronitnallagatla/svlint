FROM rust

RUN mkdir /home/app

COPY . /home/app

CMD cd /home/app && cargo test
