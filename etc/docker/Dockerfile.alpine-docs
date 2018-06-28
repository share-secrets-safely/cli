FROM alpine_with_gpg2:stable

RUN apk --no-cache -U add curl tree git
RUN curl --progress --fail -Lo termbook.tar.gz https://github.com/Byron/termbook/releases/download/1.4.2/termbook-1.4.2-x86_64-unknown-linux-musl.tar.gz \
		&& tar xzvf termbook.tar.gz \
		&& rm termbook.tar.gz \
		&& mv termbook /usr/local/bin/
