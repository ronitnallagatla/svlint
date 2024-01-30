FROM rust

RUN mkdir /home/app

COPY . /home/app

CMD cd /home/app && make && mv svlint svlint_org && make
