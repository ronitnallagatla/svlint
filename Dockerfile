FROM rust

RUN mkdir /home/app

COPY . /home/app

CMD ["ls"]
