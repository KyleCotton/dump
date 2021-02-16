FROM rustlang/rust:nightly
MAINTAINER Kyle Cotton <kyle@cybersaur.io>

WORKDIR /usr/src/sdp-backend
EXPOSE 8000
COPY . .
RUN cargo install --path .
CMD ROCKET_PORT=$PORT sdp-backend
