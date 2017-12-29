FROM rust:latest

RUN apt-get update
RUN apt-get install -y autoconf

ENV GETTEXT_VERSION=0.19.3

RUN curl -sL https://ftp.gnu.org/gnu/gettext/gettext-${GETTEXT_VERSION}.tar.gz -o /gettext-${GETTEXT_VERSION}.tar.gz
RUN tar -xf /gettext-${GETTEXT_VERSION}.tar.gz
RUN cd /gettext-${GETTEXT_VERSION} && ./configure --without-emacs --disable-java --disable-c++ --enable-fast-install > /dev/null
RUN cd /gettext-${GETTEXT_VERSION} make -j2 > /dev/null && make install > /dev/null 

ENV LD_LIBRARY_PATH=/usr/local/lib
