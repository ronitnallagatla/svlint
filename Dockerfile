FROM rust

RUN mkdir /home/app

COPY . /home/app

RUN cd /home/app

CMD ["ls"]
