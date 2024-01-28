FROM rust

RUN mkdir /home/app

COPY . /home/app

RUN cd /home/app/ && ls

CMD ["make"]