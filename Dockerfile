FROM rust:1.60.0
RUN echo 'debconf debconf/frontend select Noninteractive' | debconf-set-selections
RUN mkdir /bdk-services-code
WORKDIR /bdk-services-code
COPY . /bdk-services-code
RUN cargo install --path .