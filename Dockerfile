FROM rust

RUN mkdir svlint

COPY . /svlint

RUN cd svlint

CMD ["make"]